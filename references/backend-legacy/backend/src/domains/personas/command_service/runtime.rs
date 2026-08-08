use super::*;

impl PersonaCommandService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_identity_trace_manual(
        &self,
        identity_type: &str,
        identity_value: &str,
        requested_source: &str,
    ) -> Result<PersonaIdentity, PersonaCommandServiceError> {
        let observation = self
            .capture_manual_at(
                "PERSONA_RECORD_MUTATION",
                Utc::now(),
                json!({
                    "identity_type": identity_type,
                    "identity_value": identity_value,
                    "source": requested_source,
                }),
                format!("persona-identity://trace/{identity_type}/{identity_value}"),
                json!({
                    "captured_by": "persona_service.create_identity_trace_manual",
                    "operation": "create_identity_trace_manual",
                    "requested_source": requested_source,
                }),
            )
            .await?;

        Ok(PersonaIdentityStore::new(self.pool.clone())
            .create_unattached_with_observation(
                identity_type,
                identity_value,
                &manual_record_source(requested_source, &observation.observation_id),
                &observation.observation_id,
            )
            .await?)
    }

    pub async fn assign_identity_trace_manual(
        &self,
        identity_id: &str,
        persona_id: &str,
    ) -> Result<PersonaIdentity, PersonaCommandServiceError> {
        let observation = self
            .capture_manual_at(
                "PERSONA_RECORD_MUTATION",
                Utc::now(),
                json!({
                    "identity_id": identity_id,
                    "persona_id": persona_id,
                    "action": "attach_identity_trace",
                }),
                format!("persona-identity://trace/{identity_id}/assignment"),
                json!({
                    "captured_by": "persona_service.assign_identity_trace_manual",
                    "operation": "assign_identity_trace_manual",
                }),
            )
            .await?;

        Ok(PersonaIdentityStore::new(self.pool.clone())
            .attach_to_persona_with_observation(
                identity_id,
                persona_id,
                &observation.observation_id,
            )
            .await?)
    }

    pub async fn upsert_persona_identity_manual(
        &self,
        persona_id: &str,
        identity_type: &str,
        identity_value: &str,
        requested_source: &str,
    ) -> Result<PersonaIdentity, PersonaCommandServiceError> {
        let observation = self
            .capture_manual_at(
                "PERSONA_RECORD_MUTATION",
                Utc::now(),
                json!({
                    "persona_id": persona_id,
                    "identity_type": identity_type,
                    "identity_value": identity_value,
                    "source": requested_source,
                }),
                format!("persona://{persona_id}/identities/{identity_type}"),
                json!({
                    "captured_by": "persona_service.upsert_persona_identity_manual",
                    "operation": "upsert_persona_identity_manual",
                    "requested_source": requested_source,
                }),
            )
            .await?;

        Ok(PersonaIdentityStore::new(self.pool.clone())
            .upsert_with_observation(
                persona_id,
                identity_type,
                identity_value,
                &manual_record_source(requested_source, &observation.observation_id),
                &observation.observation_id,
            )
            .await?)
    }

    pub async fn delete_persona_identity_manual(
        &self,
        persona_id: &str,
        identity_id: &str,
    ) -> Result<bool, PersonaCommandServiceError> {
        let observation = self
            .capture_manual_at(
                "PERSONA_RECORD_MUTATION",
                Utc::now(),
                json!({
                    "persona_id": persona_id,
                    "identity_id": identity_id,
                    "action": "delete_identity",
                }),
                format!("persona://{persona_id}/identities/{identity_id}/delete"),
                json!({
                    "captured_by": "persona_service.delete_persona_identity_manual",
                    "operation": "delete_persona_identity_manual",
                }),
            )
            .await?;

        Ok(PersonaIdentityStore::new(self.pool.clone())
            .delete_with_observation(persona_id, identity_id, &observation.observation_id)
            .await?)
    }

    pub async fn assign_role_manual(
        &self,
        persona_id: &str,
        role: &str,
    ) -> Result<PersonaRole, PersonaCommandServiceError> {
        let observation = self
            .capture_manual_at(
                "PERSONA_RECORD_MUTATION",
                Utc::now(),
                json!({
                    "persona_id": persona_id,
                    "role": role,
                    "action": "assign_role",
                }),
                format!("persona://{persona_id}/roles/{role}"),
                json!({
                    "captured_by": "persona_service.assign_role_manual",
                    "operation": "assign_role_manual",
                }),
            )
            .await?;

        Ok(PersonaRoleStore::new(self.pool.clone())
            .assign_with_observation(persona_id, role, None, Some(&observation.observation_id))
            .await?)
    }

    pub async fn remove_role_manual(
        &self,
        persona_id: &str,
        role: &str,
    ) -> Result<bool, PersonaCommandServiceError> {
        let observation = self
            .capture_manual_at(
                "PERSONA_RECORD_MUTATION",
                Utc::now(),
                json!({
                    "persona_id": persona_id,
                    "role": role,
                    "action": "remove_role",
                }),
                format!("persona://{persona_id}/roles/{role}/delete"),
                json!({
                    "captured_by": "persona_service.remove_role_manual",
                    "operation": "remove_role_manual",
                }),
            )
            .await?;

        Ok(PersonaRoleStore::new(self.pool.clone())
            .remove_with_observation(persona_id, role, Some(&observation.observation_id))
            .await?)
    }

    pub async fn upsert_persona_interaction_context_manual(
        &self,
        persona: &NewPersonaInteractionContext,
    ) -> Result<PersonaInteractionContext, PersonaCommandServiceError> {
        let observation = self
            .capture_manual_at(
                "PERSONA_RECORD_MUTATION",
                Utc::now(),
                json!({
                    "persona_id": persona.source_persona_id,
                    "interaction_context_id": persona.interaction_context_id,
                    "name": persona.name,
                    "context": persona.context,
                    "default_tone": persona.default_tone,
                    "default_language": persona.default_language,
                    "preferred_channel": persona.preferred_channel,
                    "action": "upsert_interaction_context",
                }),
                format!(
                    "persona://{}/interaction-contexts/{}",
                    persona.source_persona_id, persona.interaction_context_id
                ),
                json!({
                    "captured_by": "persona_service.upsert_persona_interaction_context_manual",
                    "operation": "upsert_persona_interaction_context_manual",
                }),
            )
            .await?;

        Ok(PersonaInteractionContextStore::new(self.pool.clone())
            .upsert_with_observation(
                persona,
                Some(&format!("observation:{}", observation.observation_id)),
                &observation.observation_id,
            )
            .await?)
    }

    pub async fn delete_persona_interaction_context_manual(
        &self,
        source_persona_id: &str,
        interaction_context_id: &str,
    ) -> Result<bool, PersonaCommandServiceError> {
        let observation = self
            .capture_manual_at(
                "PERSONA_RECORD_MUTATION",
                Utc::now(),
                json!({
                    "persona_id": source_persona_id,
                    "interaction_context_id": interaction_context_id,
                    "action": "delete_interaction_context",
                }),
                format!(
                    "persona://{source_persona_id}/interaction-contexts/{interaction_context_id}/delete"
                ),
                json!({
                    "captured_by": "persona_service.delete_persona_interaction_context_manual",
                    "operation": "delete_persona_interaction_context_manual",
                }),
            )
            .await?;

        Ok(PersonaInteractionContextStore::new(self.pool.clone())
            .delete_with_observation(
                source_persona_id,
                interaction_context_id,
                Some(&format!("observation:{}", observation.observation_id)),
                &observation.observation_id,
            )
            .await?)
    }

    pub async fn upsert_persona_fact_manual(
        &self,
        persona_id: &str,
        fact_type: &str,
        value: &str,
        requested_source: &str,
        confidence: f64,
    ) -> Result<PersonaFact, PersonaCommandServiceError> {
        let observation = self
            .capture_manual_at(
                "PERSONA_RECORD_MUTATION",
                Utc::now(),
                json!({
                    "persona_id": persona_id,
                    "fact_type": fact_type,
                    "value": value,
                    "source": requested_source,
                    "confidence": confidence,
                }),
                format!("persona://{persona_id}/facts/{fact_type}"),
                json!({
                    "captured_by": "persona_service.upsert_persona_fact_manual",
                    "operation": "upsert_persona_fact_manual",
                    "requested_source": requested_source,
                }),
            )
            .await?;

        Ok(PersonaFactStore::new(self.pool.clone())
            .upsert_with_observation(
                persona_id,
                fact_type,
                value,
                &format!("observation:{}", observation.observation_id),
                confidence,
                &observation.observation_id,
            )
            .await?)
    }

    pub async fn upsert_persona_memory_card_manual(
        &self,
        persona_id: &str,
        title: &str,
        description: &str,
        requested_source: &str,
        importance: i16,
    ) -> Result<PersonaMemoryCard, PersonaCommandServiceError> {
        let observation = self
            .capture_manual_at(
                "PERSONA_MEMORY_CARD",
                Utc::now(),
                json!({
                    "persona_id": persona_id,
                    "title": title,
                    "description": description,
                    "source": requested_source,
                    "importance": importance,
                }),
                format!("persona://{persona_id}/memory-cards/{title}"),
                json!({
                    "captured_by": "persona_service.upsert_persona_memory_card_manual",
                    "operation": "upsert_persona_memory_card_manual",
                    "requested_source": requested_source,
                }),
            )
            .await?;

        Ok(PersonaMemoryCardStore::new(self.pool.clone())
            .upsert_with_observation(
                persona_id,
                title,
                description,
                &format!("observation:{}", observation.observation_id),
                importance,
                &observation.observation_id,
            )
            .await?)
    }

    pub async fn upsert_persona_preference_manual(
        &self,
        persona_id: &str,
        preference_type: &str,
        value: &str,
        requested_source: &str,
    ) -> Result<PersonaPreference, PersonaCommandServiceError> {
        let observation = self
            .capture_manual_at(
                "PERSONA_RECORD_MUTATION",
                Utc::now(),
                json!({
                    "persona_id": persona_id,
                    "preference_type": preference_type,
                    "value": value,
                    "source": requested_source,
                }),
                format!("persona://{persona_id}/preferences/{preference_type}"),
                json!({
                    "captured_by": "persona_service.upsert_persona_preference_manual",
                    "operation": "upsert_persona_preference_manual",
                    "requested_source": requested_source,
                }),
            )
            .await?;

        Ok(PersonaPreferenceStore::new(self.pool.clone())
            .upsert_with_observation(
                persona_id,
                preference_type,
                value,
                &format!("observation:{}", observation.observation_id),
                &observation.observation_id,
            )
            .await?)
    }

    pub async fn add_relationship_event_manual(
        &self,
        event: &NewRelationshipEvent,
    ) -> Result<RelationshipEvent, PersonaCommandServiceError> {
        let observation = self
            .capture_manual_at(
                "PERSONA_RECORD_MUTATION",
                event.occurred_at,
                json!({
                    "persona_id": event.persona_id,
                    "event_type": event.event_type,
                    "title": event.title,
                    "description": event.description,
                    "occurred_at": event.occurred_at,
                    "requested_source": event.source,
                    "related_entity_id": event.related_entity_id,
                    "related_entity_kind": event.related_entity_kind,
                }),
                format!("persona://{}/timeline", event.persona_id),
                json!({
                    "captured_by": "persona_service.add_relationship_event_manual",
                    "operation": "add_relationship_event_manual",
                    "requested_source": event.source,
                }),
            )
            .await?;

        Ok(RelationshipEventStore::new(self.pool.clone())
            .add_with_observation(
                &NewRelationshipEvent {
                    persona_id: event.persona_id.clone(),
                    event_type: event.event_type.clone(),
                    title: event.title.clone(),
                    description: event.description.clone(),
                    occurred_at: event.occurred_at,
                    source: format!("observation:{}", observation.observation_id),
                    related_entity_id: event.related_entity_id.clone(),
                    related_entity_kind: event.related_entity_kind.clone(),
                },
                &observation.observation_id,
            )
            .await?)
    }

    pub async fn apply_enrichment_manual(
        &self,
        persona_id: &str,
        result_id: &str,
    ) -> Result<(), PersonaCommandServiceError> {
        let observation = self
            .capture_manual_at(
                "REVIEW_TRANSITION",
                Utc::now(),
                json!({
                    "persona_id": persona_id,
                    "result_id": result_id,
                    "operation": "enrichment_apply",
                }),
                format!("persona://{persona_id}/enrichment/{result_id}/apply"),
                json!({
                    "captured_by": "persona_service.apply_enrichment_manual",
                    "operation": "apply_enrichment_manual",
                }),
            )
            .await?;

        EnrichmentResultStore::new(self.pool.clone())
            .apply_with_observation(
                result_id,
                Some(&observation.observation_id),
                Some(json!({
                    "captured_by": "persona_service.apply_enrichment_manual",
                    "operation": "apply_enrichment_manual",
                })),
            )
            .await?;
        Ok(())
    }

    pub async fn reject_enrichment_manual(
        &self,
        persona_id: &str,
        result_id: &str,
    ) -> Result<(), PersonaCommandServiceError> {
        let observation = self
            .capture_manual_at(
                "REVIEW_TRANSITION",
                Utc::now(),
                json!({
                    "persona_id": persona_id,
                    "result_id": result_id,
                    "operation": "enrichment_reject",
                }),
                format!("persona://{persona_id}/enrichment/{result_id}/reject"),
                json!({
                    "captured_by": "persona_service.reject_enrichment_manual",
                    "operation": "reject_enrichment_manual",
                }),
            )
            .await?;

        EnrichmentResultStore::new(self.pool.clone())
            .reject_with_observation(
                result_id,
                Some(&observation.observation_id),
                Some(json!({
                    "captured_by": "persona_service.reject_enrichment_manual",
                    "operation": "reject_enrichment_manual",
                })),
            )
            .await?;
        Ok(())
    }

    pub async fn toggle_watchlist_manual(
        &self,
        persona_id: &str,
    ) -> Result<bool, PersonaCommandServiceError> {
        let observation = self
            .capture_manual_at(
                "PERSONA_MUTATION",
                Utc::now(),
                json!({
                    "persona_id": persona_id,
                    "action": "toggle_watchlist",
                }),
                format!("persona://{persona_id}/watchlist"),
                json!({
                    "captured_by": "persona_service.toggle_watchlist_manual",
                    "operation": "toggle_watchlist_manual",
                }),
            )
            .await?;

        Ok(PersonaHealthStore::new(self.pool.clone())
            .toggle_watchlist_with_observation(
                persona_id,
                &format!("observation:{}", observation.observation_id),
                &observation.observation_id,
            )
            .await?)
    }

    pub async fn fingerprint_persona_manual(
        &self,
        persona_id: &str,
        person_messages: &[PersonaMessage],
    ) -> Result<Value, PersonaCommandServiceError> {
        let mut fingerprint = PersonaIntelligenceService::heuristic_fingerprint(person_messages);
        if fingerprint.trust_score.is_none() {
            fingerprint.trust_score = Some(50);
        }

        let observation = self
            .capture_manual_at(
                "PERSONA_MUTATION",
                Utc::now(),
                json!({
                    "persona_id": persona_id,
                    "action": "fingerprint_enrichment",
                    "detected_language": fingerprint.detected_language,
                    "typical_tone": fingerprint.typical_tone,
                    "trust_score": fingerprint.trust_score,
                    "avg_response_hours": fingerprint.avg_response_hours,
                    "writing_style": fingerprint.writing_style,
                }),
                format!("persona://{persona_id}/fingerprint"),
                json!({
                    "captured_by": "persona_service.fingerprint_persona_manual",
                    "operation": "fingerprint_persona_manual",
                }),
            )
            .await?;

        PersonaEnrichmentStore::new(self.pool.clone())
            .enrich_persona_with_observation(persona_id, &fingerprint, &observation.observation_id)
            .await?;

        Ok(json!({
            "enriched": true,
            "fingerprint": fingerprint,
        }))
    }

    pub async fn toggle_favorite_manual(
        &self,
        persona_id: &str,
    ) -> Result<bool, PersonaCommandServiceError> {
        let observation = self
            .capture_manual_at(
                "PERSONA_MUTATION",
                Utc::now(),
                json!({
                    "persona_id": persona_id,
                    "action": "toggle_favorite",
                }),
                format!("persona://{persona_id}/favorite"),
                json!({
                    "captured_by": "persona_service.toggle_favorite_manual",
                    "operation": "toggle_favorite_manual",
                }),
            )
            .await?;

        Ok(PersonaEnrichmentStore::new(self.pool.clone())
            .toggle_favorite_with_observation(
                persona_id,
                &format!("observation:{}", observation.observation_id),
                &observation.observation_id,
            )
            .await?)
    }

    pub async fn set_notes_manual(
        &self,
        persona_id: &str,
        notes: &str,
    ) -> Result<(), PersonaCommandServiceError> {
        let observation = self
            .capture_manual_at(
                "PERSONA_MEMORY_CARD",
                Utc::now(),
                json!({
                    "persona_id": persona_id,
                    "title": "Persona notes",
                    "body": notes,
                }),
                format!("persona://{persona_id}/notes"),
                json!({
                    "captured_by": "persona_service.set_notes_manual",
                    "operation": "set_notes_manual",
                }),
            )
            .await?;

        PersonaEnrichmentStore::new(self.pool.clone())
            .set_notes_with_observation(
                persona_id,
                notes,
                &format!("observation:{}", observation.observation_id),
                &observation.observation_id,
            )
            .await?;
        Ok(())
    }

    pub async fn set_owner_persona_manual(
        &self,
        persona_id: &str,
    ) -> Result<Persona, PersonaCommandServiceError> {
        let observation = self
            .capture_manual_at(
                "PERSONA_MUTATION",
                Utc::now(),
                json!({
                    "persona_id": persona_id,
                    "operation": "set_owner_persona",
                }),
                format!("persona://{persona_id}/owner"),
                json!({
                    "captured_by": "persona_service.set_owner_persona_manual",
                    "operation": "set_owner_persona_manual",
                }),
            )
            .await?;

        Ok(PersonaProjectionStore::new(self.pool.clone())
            .set_owner_persona_with_observation(persona_id, &observation.observation_id)
            .await?)
    }

    pub async fn update_persona_manual(
        &self,
        persona_id: &str,
        display_name: Option<&str>,
        set_self: bool,
    ) -> Result<Persona, PersonaCommandServiceError> {
        let observation = self
            .capture_manual_at(
                "PERSONA_MUTATION",
                Utc::now(),
                json!({
                    "persona_id": persona_id,
                    "display_name": display_name,
                    "is_self": set_self,
                }),
                format!("persona://{persona_id}/update"),
                json!({
                    "captured_by": "persona_service.update_persona_manual",
                    "operation": "update_persona_manual",
                }),
            )
            .await?;

        Ok(PersonaProjectionStore::new(self.pool.clone())
            .update_persona_with_observation(
                persona_id,
                display_name,
                set_self,
                &observation.observation_id,
            )
            .await?)
    }

    pub async fn set_persona_address_book_membership_manual(
        &self,
        persona_id: &str,
        is_address_book: bool,
    ) -> Result<Persona, PersonaCommandServiceError> {
        let observation = self
            .capture_manual_at(
                "PERSONA_MUTATION",
                Utc::now(),
                json!({
                    "persona_id": persona_id,
                    "is_address_book": is_address_book,
                    "operation": "set_address_book_membership",
                }),
                format!("persona://{persona_id}/address-book-membership"),
                json!({
                    "captured_by": "persona_service.set_persona_address_book_membership_manual",
                    "operation": "set_persona_address_book_membership_manual",
                }),
            )
            .await?;

        Ok(PersonaProjectionStore::new(self.pool.clone())
            .set_address_book_membership_with_observation(
                persona_id,
                is_address_book,
                &observation.observation_id,
            )
            .await?)
    }

    pub async fn upsert_persona_from_address_book_entry(
        &self,
        command: ProviderAddressBookEntryPersonaCommand,
    ) -> Result<Persona, PersonaCommandServiceError> {
        let observation = self
            .capture_local_runtime_at(
                "PERSONA_MUTATION",
                Utc::now(),
                json!({
                    "source_account_id": &command.source_account_id,
                    "provider_address_book_entry_id": &command.provider_address_book_entry_id,
                    "display_name": &command.display_name,
                    "primary_email": &command.primary_email,
                    "additional_email_count": command.additional_emails.len(),
                    "phone_number_count": command.phone_numbers.len(),
                    "operation": "upsert_persona_from_address_book_entry",
                }),
                format!(
                    "provider-address-book-entry://{}/{}",
                    command.source_account_id, command.provider_address_book_entry_id
                ),
                json!({
                    "captured_by": "persona_service.upsert_persona_from_address_book_entry",
                    "operation": "upsert_persona_from_address_book_entry",
                    "source_account_id": &command.source_account_id,
                }),
            )
            .await?;

        let identity_store = PersonaIdentityStore::new(self.pool.clone());
        let phone_identities = normalized_address_book_phone_numbers(&command.phone_numbers);
        let existing_phone_person_id =
            first_existing_identity_person_id(&identity_store, "phone", &phone_identities).await?;
        let existing_provider_link_person_id = existing_provider_address_book_link_person_id(
            &self.pool,
            &command.source_account_id,
            &command.provider_address_book_entry_id,
        )
        .await?;
        let fallback_person_id = existing_phone_person_id
            .clone()
            .or(existing_provider_link_person_id)
            .unwrap_or_else(|| {
                provider_address_book_entry_persona_id(
                    &command.source_account_id,
                    &command.provider_address_book_entry_id,
                )
            });

        let persona = if let (Some(existing_persona_id), Some(primary_email)) = (
            existing_phone_person_id.as_deref(),
            command.primary_email.as_deref(),
        ) {
            PersonaProjectionStore::new(self.pool.clone())
                .upsert_address_book_email_for_existing_persona(
                    existing_persona_id,
                    command.display_name.as_deref(),
                    primary_email,
                )
                .await?
        } else {
            PersonaProjectionStore::new(self.pool.clone())
                .upsert_address_book_persona(
                    command.display_name.as_deref(),
                    command.primary_email.as_deref(),
                    &fallback_person_id,
                )
                .await?
        };
        for email_address in command.additional_emails {
            identity_store
                .upsert_with_observation(
                    &persona.persona_id,
                    "email",
                    &email_address,
                    "address_book_sync",
                    &observation.observation_id,
                )
                .await?;
        }
        for phone_number in phone_identities {
            identity_store
                .upsert_with_observation(
                    &persona.persona_id,
                    "phone",
                    &phone_number,
                    "address_book_sync",
                    &observation.observation_id,
                )
                .await?;
        }

        Ok(persona)
    }

    pub async fn review_identity_candidate_manual(
        &self,
        command: &PersonaIdentityReviewCommand,
    ) -> Result<PersonaIdentityReviewCommandResult, PersonaCommandServiceError> {
        let observation = self
            .capture_manual_at(
                "REVIEW_TRANSITION",
                Utc::now(),
                json!({
                    "identity_candidate_id": command.identity_candidate_id,
                    "command_id": command.command_id,
                    "review_state": command.review_state.as_str(),
                    "actor_id": command.actor_id,
                    "operation": "identity_candidate_review",
                }),
                format!(
                    "identity-candidate://{}/review/{}",
                    command.identity_candidate_id, command.command_id
                ),
                json!({
                    "captured_by": "persona_service.review_identity_candidate_manual",
                    "operation": "review_identity_candidate_manual",
                }),
            )
            .await?;

        Ok(PersonaIdentityReviewStore::new(self.pool.clone())
            .set_review_state_with_observation(
                command,
                Some(&observation.observation_id),
                Some(json!({
                    "captured_by": "persona_service.review_identity_candidate_manual",
                    "operation": "review_identity_candidate_manual",
                })),
            )
            .await?)
    }

    pub async fn review_dossier_manual(
        &self,
        persona_id: &str,
        review_state: DossierReviewState,
    ) -> Result<DossierSnapshot, PersonaCommandServiceError> {
        let observation = self
            .capture_manual_at(
                "REVIEW_TRANSITION",
                Utc::now(),
                json!({
                    "persona_id": persona_id,
                    "review_state": review_state.as_str(),
                    "operation": "dossier_review",
                }),
                format!("persona://{persona_id}/dossier/review"),
                json!({
                    "captured_by": "persona_service.review_dossier_manual",
                    "operation": "review_dossier_manual",
                }),
            )
            .await?;

        Ok(PersonaInvestigator::new(self.pool.clone())
            .review_dossier_snapshot_with_observation(
                persona_id,
                review_state,
                Some(&observation.observation_id),
                Some(json!({
                    "captured_by": "persona_service.review_dossier_manual",
                    "operation": "review_dossier_manual",
                })),
            )
            .await?)
    }

    async fn capture_manual_at(
        &self,
        kind: &str,
        observed_at: DateTime<Utc>,
        payload: Value,
        source_ref: String,
        provenance: Value,
    ) -> Result<makosh_observations_api::models::Observation, PersonaCommandServiceError> {
        Ok(ObservationStore::new(self.pool.clone())
            .capture(
                &NewObservation::new(
                    kind,
                    ObservationOriginKind::Manual,
                    observed_at,
                    payload,
                    source_ref,
                )
                .provenance(provenance),
            )
            .await?)
    }

    async fn capture_local_runtime_at(
        &self,
        kind: &str,
        observed_at: DateTime<Utc>,
        payload: Value,
        source_ref: String,
        provenance: Value,
    ) -> Result<makosh_observations_api::models::Observation, PersonaCommandServiceError> {
        Ok(ObservationStore::new(self.pool.clone())
            .capture(
                &NewObservation::new(
                    kind,
                    ObservationOriginKind::LocalRuntime,
                    observed_at,
                    payload,
                    source_ref,
                )
                .provenance(provenance),
            )
            .await?)
    }
}
