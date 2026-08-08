//! Exact descriptor/grant/distribution intersection for managed runtime resources.

use makosh_runtime_protocol::v1::{
    DistributionArtifactKindV1, DistributionManifestArtifactV1, DistributionManifestV1,
    ModuleDescriptorV1, ModuleKindV1, RuntimeArtifactUseV1, capability_request_v1,
};

pub struct RuntimeArtifactRequirementV1 {
    artifact: DistributionManifestArtifactV1,
    use_kind: RuntimeArtifactUseV1,
}

impl RuntimeArtifactRequirementV1 {
    #[must_use]
    pub fn artifact(&self) -> &DistributionManifestArtifactV1 {
        &self.artifact
    }

    #[must_use]
    pub fn use_kind(&self) -> RuntimeArtifactUseV1 {
        self.use_kind
    }
}

pub struct ManagedRuntimeRequirementsV1 {
    runtime_artifacts: Vec<RuntimeArtifactRequirementV1>,
    state_layout_revision: Option<u32>,
}

impl ManagedRuntimeRequirementsV1 {
    #[must_use]
    pub fn runtime_artifacts(&self) -> &[RuntimeArtifactRequirementV1] {
        &self.runtime_artifacts
    }

    #[must_use]
    pub fn state_layout_revision(&self) -> Option<u32> {
        self.state_layout_revision
    }
}

pub fn select(
    descriptor: &ModuleDescriptorV1,
    granted_capability_ids: &[String],
    manifest: &DistributionManifestV1,
) -> Result<ManagedRuntimeRequirementsV1, String> {
    let module_kind = ModuleKindV1::try_from(descriptor.module_kind)
        .ok()
        .filter(|kind| {
            matches!(
                kind,
                ModuleKindV1::Integration | ModuleKindV1::Workflow | ModuleKindV1::Engine
            )
        })
        .ok_or_else(|| "managed runtime kind cannot request runtime resources".to_owned())?;
    if granted_capability_ids
        .windows(2)
        .any(|pair| pair[0] >= pair[1])
    {
        return Err("managed runtime grants are not exact ordered identities".to_owned());
    }
    for capability_id in granted_capability_ids {
        if descriptor
            .capabilities
            .binary_search_by(|candidate| candidate.capability_id.as_str().cmp(capability_id))
            .is_err()
        {
            return Err("managed runtime grant is absent from exact descriptor".to_owned());
        }
    }

    let mut runtime_artifacts = Vec::new();
    let mut state_layout_revision = None;
    for capability in &descriptor.capabilities {
        if granted_capability_ids
            .binary_search_by(|candidate| candidate.as_str().cmp(&capability.capability_id))
            .is_err()
        {
            continue;
        }
        for request in &capability.requests {
            match request.request.as_ref() {
                Some(capability_request_v1::Request::RuntimeArtifact(request)) => {
                    let use_kind = RuntimeArtifactUseV1::try_from(request.r#use)
                        .ok()
                        .filter(|value| *value != RuntimeArtifactUseV1::Unspecified)
                        .ok_or_else(|| "managed runtime artifact use is unsupported".to_owned())?;
                    let artifact = manifest
                        .artifacts
                        .binary_search_by(|candidate| {
                            candidate.artifact_id.as_str().cmp(&request.artifact_id)
                        })
                        .ok()
                        .map(|index| &manifest.artifacts[index])
                        .ok_or_else(|| {
                            "managed runtime artifact is absent from distribution".to_owned()
                        })?;
                    if artifact.artifact_kind != distribution_kind(use_kind) as i32
                        || artifact.bound_module_id != descriptor.module_id
                    {
                        return Err("managed runtime artifact binding is invalid".to_owned());
                    }
                    runtime_artifacts.push(RuntimeArtifactRequirementV1 {
                        artifact: artifact.clone(),
                        use_kind,
                    });
                }
                Some(capability_request_v1::Request::IntegrationState(request)) => {
                    if module_kind != ModuleKindV1::Integration
                        || request.state_layout_revision == 0
                        || state_layout_revision
                            .is_some_and(|revision| revision != request.state_layout_revision)
                    {
                        return Err(
                            "managed integration state layout request is ambiguous".to_owned()
                        );
                    }
                    state_layout_revision = Some(request.state_layout_revision);
                }
                _ => {}
            }
        }
    }

    runtime_artifacts.sort_by(|left, right| {
        left.artifact
            .artifact_id
            .cmp(&right.artifact.artifact_id)
            .then_with(|| (left.use_kind as i32).cmp(&(right.use_kind as i32)))
    });
    for pair in runtime_artifacts.windows(2) {
        if pair[0].artifact.artifact_id == pair[1].artifact.artifact_id
            && pair[0].use_kind != pair[1].use_kind
        {
            return Err("managed runtime artifact use is ambiguous".to_owned());
        }
    }
    runtime_artifacts.dedup_by(|left, right| {
        left.artifact.artifact_id == right.artifact.artifact_id && left.use_kind == right.use_kind
    });
    Ok(ManagedRuntimeRequirementsV1 {
        runtime_artifacts,
        state_layout_revision,
    })
}

const fn distribution_kind(use_kind: RuntimeArtifactUseV1) -> DistributionArtifactKindV1 {
    match use_kind {
        RuntimeArtifactUseV1::NativeDynamicLibrary => {
            DistributionArtifactKindV1::ModuleRuntimeNativeDependency
        }
        RuntimeArtifactUseV1::NativeExecutable => {
            DistributionArtifactKindV1::ModuleRuntimeNativeExecutable
        }
        RuntimeArtifactUseV1::ReadOnlyData => DistributionArtifactKindV1::ModuleRuntimeReadOnlyData,
        RuntimeArtifactUseV1::Unspecified => DistributionArtifactKindV1::Unspecified,
    }
}

#[cfg(test)]
mod tests {
    use super::select;
    use makosh_runtime_protocol::v1::{
        CapabilityCriticalityV1, CapabilityDescriptorV1, CapabilityRequestV1,
        DistributionArtifactKindV1, DistributionManifestArtifactV1, DistributionManifestV1,
        IntegrationStateRequestV1, ModuleDescriptorV1, ModuleKindV1, RuntimeArtifactRequestV1,
        RuntimeArtifactUseV1, capability_request_v1,
    };

    #[test]
    fn selects_only_granted_exact_module_artifacts_and_one_state_layout() {
        let descriptor = descriptor();
        let manifest = manifest("makosh-telegram-runtime");

        let none = select(&descriptor, &[], &manifest).expect("no grants");
        assert!(none.runtime_artifacts().is_empty());
        assert_eq!(none.state_layout_revision(), None);

        let requirements = select(&descriptor, &["telegram.runtime.v1".to_owned()], &manifest)
            .expect("granted runtime");
        assert_eq!(requirements.runtime_artifacts().len(), 1);
        assert_eq!(
            requirements.runtime_artifacts()[0].artifact().artifact_id,
            "telegram.tdjson.v1"
        );
        assert_eq!(requirements.state_layout_revision(), Some(1));
    }

    #[test]
    fn rejects_unknown_grants_and_cross_module_artifact_binding() {
        let descriptor = descriptor();
        assert!(
            select(
                &descriptor,
                &["telegram.unknown.v1".to_owned()],
                &manifest("makosh-telegram-runtime"),
            )
            .is_err()
        );
        assert!(
            select(
                &descriptor,
                &["telegram.runtime.v1".to_owned()],
                &manifest("makosh-other-runtime"),
            )
            .is_err()
        );
    }

    #[test]
    fn selects_exact_workflow_executable_and_model_data_but_rejects_domains() {
        let mut descriptor = descriptor();
        descriptor.module_id = "makosh-attachment-text-extraction-runtime".to_owned();
        descriptor.owner_id = "attachment_text_extraction".to_owned();
        descriptor.module_kind = ModuleKindV1::Workflow as i32;
        descriptor.capabilities[0].capability_id =
            "attachment_text_extraction.ocr_runtime.v1".to_owned();
        descriptor.capabilities[0].requests = vec![
            runtime_artifact(
                "attachment_text_extraction.ocr.eng.v1",
                RuntimeArtifactUseV1::ReadOnlyData,
            ),
            runtime_artifact(
                "attachment_text_extraction.ocr.runner.v1",
                RuntimeArtifactUseV1::NativeExecutable,
            ),
            runtime_artifact(
                "attachment_text_extraction.ocr.rus.v1",
                RuntimeArtifactUseV1::ReadOnlyData,
            ),
        ];
        let manifest = DistributionManifestV1 {
            artifacts: vec![
                runtime_manifest_artifact(
                    "attachment_text_extraction.ocr.eng.v1",
                    DistributionArtifactKindV1::ModuleRuntimeReadOnlyData,
                    &descriptor.module_id,
                ),
                runtime_manifest_artifact(
                    "attachment_text_extraction.ocr.runner.v1",
                    DistributionArtifactKindV1::ModuleRuntimeNativeExecutable,
                    &descriptor.module_id,
                ),
                runtime_manifest_artifact(
                    "attachment_text_extraction.ocr.rus.v1",
                    DistributionArtifactKindV1::ModuleRuntimeReadOnlyData,
                    &descriptor.module_id,
                ),
            ],
            ..Default::default()
        };
        let grants = vec!["attachment_text_extraction.ocr_runtime.v1".to_owned()];
        let requirements = select(&descriptor, &grants, &manifest).expect("workflow resources");
        assert_eq!(requirements.runtime_artifacts().len(), 3);
        assert_eq!(
            requirements.runtime_artifacts()[1].use_kind(),
            RuntimeArtifactUseV1::NativeExecutable
        );
        assert_eq!(requirements.state_layout_revision(), None);

        descriptor.module_kind = ModuleKindV1::Domain as i32;
        assert!(select(&descriptor, &grants, &manifest).is_err());
    }

    fn runtime_artifact(id: &str, use_kind: RuntimeArtifactUseV1) -> CapabilityRequestV1 {
        CapabilityRequestV1 {
            request: Some(capability_request_v1::Request::RuntimeArtifact(
                RuntimeArtifactRequestV1 {
                    artifact_id: id.to_owned(),
                    r#use: use_kind as i32,
                },
            )),
        }
    }

    fn runtime_manifest_artifact(
        id: &str,
        kind: DistributionArtifactKindV1,
        bound_module_id: &str,
    ) -> DistributionManifestArtifactV1 {
        DistributionManifestArtifactV1 {
            artifact_kind: kind as i32,
            artifact_id: id.to_owned(),
            bound_module_id: bound_module_id.to_owned(),
            ..Default::default()
        }
    }

    fn descriptor() -> ModuleDescriptorV1 {
        ModuleDescriptorV1 {
            descriptor_major: 1,
            descriptor_revision: 1,
            module_id: "makosh-telegram-runtime".to_owned(),
            owner_id: "telegram".to_owned(),
            module_kind: ModuleKindV1::Integration as i32,
            module_version: "1".to_owned(),
            build_id: "build-1".to_owned(),
            capabilities: vec![CapabilityDescriptorV1 {
                capability_id: "telegram.runtime.v1".to_owned(),
                capability_revision: 1,
                criticality: CapabilityCriticalityV1::Required as i32,
                requests: vec![
                    CapabilityRequestV1 {
                        request: Some(capability_request_v1::Request::RuntimeArtifact(
                            RuntimeArtifactRequestV1 {
                                artifact_id: "telegram.tdjson.v1".to_owned(),
                                r#use: RuntimeArtifactUseV1::NativeDynamicLibrary as i32,
                            },
                        )),
                    },
                    CapabilityRequestV1 {
                        request: Some(capability_request_v1::Request::IntegrationState(
                            IntegrationStateRequestV1 {
                                state_layout_revision: 1,
                            },
                        )),
                    },
                ],
                ..Default::default()
            }],
            ..Default::default()
        }
    }

    fn manifest(bound_module_id: &str) -> DistributionManifestV1 {
        DistributionManifestV1 {
            artifacts: vec![DistributionManifestArtifactV1 {
                artifact_kind: DistributionArtifactKindV1::ModuleRuntimeNativeDependency as i32,
                artifact_id: "telegram.tdjson.v1".to_owned(),
                bound_module_id: bound_module_id.to_owned(),
                ..Default::default()
            }],
            ..Default::default()
        }
    }
}
