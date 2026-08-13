use makosh_reviewed_person_match_candidate_promotion_assembly::materialize_reviewed_person_match_candidate_promotion_assembly_v1;
use std::path::PathBuf;

fn main() -> Result<(), String> {
    run(std::env::args().skip(1))
}

fn run(mut args: impl Iterator<Item = String>) -> Result<(), String> {
    if args.next().as_deref() != Some("assemble") {
        return Err("Reviewed Person Match Candidate Promotion assembly command is invalid".into());
    }
    let output = required(&mut args, "--output-dir")?;
    let build = required(&mut args, "--build-id")?;
    let runtime = required(&mut args, "--runtime-source")?;
    if args.next().is_some() {
        return Err(
            "Reviewed Person Match Candidate Promotion assembly arguments are invalid".into(),
        );
    }
    materialize_reviewed_person_match_candidate_promotion_assembly_v1(
        &PathBuf::from(output),
        &build,
        &PathBuf::from(runtime),
    )
    .map(|_| ())
    .map_err(|_| "Reviewed Person Match Candidate Promotion assembly failed".into())
}
fn required(args: &mut impl Iterator<Item = String>, name: &str) -> Result<String, String> {
    if args.next().as_deref() != Some(name) {
        return Err(
            "Reviewed Person Match Candidate Promotion assembly arguments are invalid".into(),
        );
    }
    args.next()
        .filter(|v| !v.is_empty())
        .ok_or_else(|| format!("{name} is required"))
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::PathBuf,
        sync::atomic::{AtomicU64, Ordering},
    };

    use super::run;

    static NEXT: AtomicU64 = AtomicU64::new(1);

    fn temp() -> PathBuf {
        std::env::temp_dir().join(format!(
            "makosh-reviewed-promotion-assembly-cli-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ))
    }

    #[test]
    fn cli_writes_canonical_fragment_and_cleans_failure() {
        let root = temp();
        fs::create_dir(&root).expect("root");
        let runtime = root.join("runtime");
        fs::write(&runtime, b"runtime").expect("runtime");
        let output = root.join("output");
        let args = [
            "assemble".to_owned(),
            "--output-dir".to_owned(),
            output.display().to_string(),
            "--build-id".to_owned(),
            "build-1".to_owned(),
            "--runtime-source".to_owned(),
            runtime.display().to_string(),
        ];
        run(args.into_iter()).expect("assemble");
        assert!(
            output
                .join("reviewed-person-match-candidate-promotion.release-artifacts.json")
                .is_file()
        );

        let failed = root.join("failed");
        let bad_args = [
            "assemble".to_owned(),
            "--output-dir".to_owned(),
            failed.display().to_string(),
            "--build-id".to_owned(),
            "build-1".to_owned(),
            "--runtime-source".to_owned(),
            root.join("missing").display().to_string(),
        ];
        assert!(run(bad_args.into_iter()).is_err());
        assert!(!failed.exists());
        fs::remove_dir_all(root).expect("cleanup");
    }
}
