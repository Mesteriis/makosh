use std::path::PathBuf;

use makosh_review_person_match_candidate_assembly::materialize_review_person_match_candidate_assembly_v1;

fn main() -> Result<(), String> {
    run(std::env::args().skip(1))
}

fn run(mut arguments: impl Iterator<Item = String>) -> Result<(), String> {
    if arguments.next().as_deref() != Some("assemble") {
        return Err("Review Person Match Candidate assembly command is invalid".to_owned());
    }
    let output = required(&mut arguments, "--output-dir")?;
    let build_id = required(&mut arguments, "--build-id")?;
    let runtime = required(&mut arguments, "--runtime-source")?;
    if arguments.next().is_some() {
        return Err("Review Person Match Candidate assembly arguments are invalid".to_owned());
    }
    materialize_review_person_match_candidate_assembly_v1(
        &PathBuf::from(output),
        &build_id,
        &PathBuf::from(runtime),
    )
    .map(|_| ())
    .map_err(|_| "Review Person Match Candidate assembly failed".to_owned())
}

fn required(arguments: &mut impl Iterator<Item = String>, name: &str) -> Result<String, String> {
    if arguments.next().as_deref() != Some(name) {
        return Err("Review Person Match Candidate assembly arguments are invalid".to_owned());
    }
    arguments
        .next()
        .filter(|value| !value.is_empty())
        .ok_or_else(|| format!("Review Person Match Candidate {name} is required"))
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
            "makosh-review-assembly-cli-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ))
    }

    #[test]
    fn cli_is_deterministic_and_failure_cleans_output() {
        let root = temp();
        fs::create_dir(&root).expect("root");
        let runtime = root.join("runtime");
        fs::write(&runtime, b"runtime").expect("runtime");
        let first = root.join("first");
        let args = [
            "assemble".to_owned(),
            "--output-dir".to_owned(),
            first.display().to_string(),
            "--build-id".to_owned(),
            "build-1".to_owned(),
            "--runtime-source".to_owned(),
            runtime.display().to_string(),
        ];
        run(args.into_iter()).expect("assemble");
        assert!(
            first
                .join("review-person-match-candidate.release-artifacts.json")
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
