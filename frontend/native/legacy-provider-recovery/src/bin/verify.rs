use std::path::PathBuf;

use makosh_legacy_provider_recovery::{
    LegacyProviderCandidateKindV1, LegacyProviderRecoveryBundleV1, LegacyProviderRecoveryErrorV1,
    LegacyProviderRecoverySecretPurposeV1, LegacyProviderRecoverySessionsV1,
};
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct VerificationReceiptV1 {
    schema_revision: u16,
    secret_validation: &'static str,
}

fn main() {
    match run() {
        Ok(receipt) => match serde_json::to_string(&receipt) {
            Ok(receipt) => println!("{receipt}"),
            Err(_) => exit(LegacyProviderRecoveryErrorV1::InvalidConfiguration),
        },
        Err(error) => exit(error),
    }
}

fn run() -> Result<VerificationReceiptV1, LegacyProviderRecoveryErrorV1> {
    let root = parse_args(std::env::args().skip(1))?;
    let sessions =
        LegacyProviderRecoverySessionsV1::new(LegacyProviderRecoveryBundleV1::open(&root)?);
    let plan = sessions.start()?;
    let icloud = handle(&plan, LegacyProviderCandidateKindV1::Icloud)?;
    let telegram = handle(&plan, LegacyProviderCandidateKindV1::TelegramUser)?;
    let imap_password = sessions.resolve_secret(
        &plan.session_id,
        icloud,
        LegacyProviderRecoverySecretPurposeV1::IcloudImapPassword,
    )?;
    let api_hash = sessions.resolve_secret(
        &plan.session_id,
        telegram,
        LegacyProviderRecoverySecretPurposeV1::TelegramApiHash,
    )?;
    let session_key = sessions.resolve_secret(
        &plan.session_id,
        telegram,
        LegacyProviderRecoverySecretPurposeV1::GeneratedTelegramSessionStoreKey,
    )?;
    if imap_password.is_empty() || api_hash.is_empty() || session_key.len() != 32 {
        return Err(LegacyProviderRecoveryErrorV1::InvalidSecret);
    }
    sessions.cancel(&plan.session_id)?;
    Ok(VerificationReceiptV1 {
        schema_revision: 1,
        secret_validation: "ok",
    })
}

fn handle(
    plan: &makosh_legacy_provider_recovery::LegacyProviderRecoveryPlanV1,
    kind: LegacyProviderCandidateKindV1,
) -> Result<&str, LegacyProviderRecoveryErrorV1> {
    plan.candidates
        .iter()
        .find(|candidate| candidate.kind == kind)
        .map(|candidate| candidate.handle.as_str())
        .ok_or(LegacyProviderRecoveryErrorV1::InvalidCatalog)
}

fn parse_args(
    mut args: impl Iterator<Item = String>,
) -> Result<PathBuf, LegacyProviderRecoveryErrorV1> {
    if args.next().as_deref() != Some("--bundle-root") {
        return Err(LegacyProviderRecoveryErrorV1::InvalidArguments);
    }
    let root = PathBuf::from(
        args.next()
            .ok_or(LegacyProviderRecoveryErrorV1::InvalidArguments)?,
    );
    if args.next().is_some() {
        return Err(LegacyProviderRecoveryErrorV1::InvalidArguments);
    }
    Ok(root)
}

fn exit(error: LegacyProviderRecoveryErrorV1) -> ! {
    eprintln!("{}", error.code());
    std::process::exit(1)
}
