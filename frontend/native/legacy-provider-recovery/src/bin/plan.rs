use std::path::PathBuf;

use makosh_legacy_provider_recovery::{
    LegacyProviderRecoveryBundleV1, LegacyProviderRecoveryErrorV1, LegacyProviderRecoverySessionsV1,
};

fn main() {
    match run() {
        Ok(plan) => match serde_json::to_string(&plan) {
            Ok(plan) => println!("{plan}"),
            Err(_) => exit(LegacyProviderRecoveryErrorV1::InvalidConfiguration),
        },
        Err(error) => exit(error),
    }
}

fn run() -> Result<
    makosh_legacy_provider_recovery::LegacyProviderRecoveryPlanV1,
    LegacyProviderRecoveryErrorV1,
> {
    let root = parse_args(std::env::args().skip(1))?;
    let bundle = LegacyProviderRecoveryBundleV1::open(&root)?;
    LegacyProviderRecoverySessionsV1::new(bundle).start()
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
