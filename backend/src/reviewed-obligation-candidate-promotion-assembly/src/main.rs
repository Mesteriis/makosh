use std::{path::PathBuf, process::ExitCode};

use makosh_reviewed_obligation_candidate_promotion_assembly::materialize_reviewed_obligation_candidate_promotion_release_assembly_v1;

const OPTIONS: [&str; 3] = ["--build-id", "--output-dir", "--runtime"];

struct Arguments {
    build_id: String,
    output_directory: PathBuf,
    runtime: PathBuf,
}

fn main() -> ExitCode {
    let Some(arguments) = arguments(std::env::args().skip(1).collect()) else {
        return fail(
            "usage: makosh-reviewed-obligation-candidate-promotion-assembly --build-id <id> \
             --output-dir <absolute-path> --runtime <absolute-path>",
        );
    };
    if materialize_reviewed_obligation_candidate_promotion_release_assembly_v1(
        &arguments.output_directory,
        &arguments.build_id,
        &arguments.runtime,
    )
    .is_err()
    {
        return fail("assembly failed");
    }
    println!("reviewed-obligation-candidate-promotion-release-assembly: ok");
    ExitCode::SUCCESS
}

fn arguments(values: Vec<String>) -> Option<Arguments> {
    if values.len() != OPTIONS.len() * 2 {
        return None;
    }
    let mut parsed = std::collections::BTreeMap::new();
    for pair in values.chunks_exact(2) {
        if !OPTIONS.contains(&pair[0].as_str())
            || pair[1].is_empty()
            || parsed.insert(pair[0].clone(), pair[1].clone()).is_some()
        {
            return None;
        }
    }
    Some(Arguments {
        build_id: parsed.remove("--build-id")?,
        output_directory: PathBuf::from(parsed.remove("--output-dir")?),
        runtime: PathBuf::from(parsed.remove("--runtime")?),
    })
}

fn fail(message: &str) -> ExitCode {
    eprintln!("{message}");
    ExitCode::FAILURE
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_each_exact_option_once() {
        let parsed = arguments(vec![
            "--runtime".to_owned(),
            "/runtime".to_owned(),
            "--build-id".to_owned(),
            "build-1".to_owned(),
            "--output-dir".to_owned(),
            "/output".to_owned(),
        ])
        .expect("arguments");
        assert_eq!(parsed.build_id, "build-1");
        assert_eq!(parsed.output_directory, PathBuf::from("/output"));
        assert_eq!(parsed.runtime, PathBuf::from("/runtime"));
    }

    #[test]
    fn rejects_duplicate_and_missing_options() {
        assert!(arguments(Vec::new()).is_none());
        assert!(
            arguments(vec![
                "--build-id".to_owned(),
                "one".to_owned(),
                "--build-id".to_owned(),
                "two".to_owned(),
                "--runtime".to_owned(),
                "/runtime".to_owned(),
            ])
            .is_none()
        );
    }
}
