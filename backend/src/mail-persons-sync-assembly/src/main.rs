use std::{collections::BTreeMap, path::PathBuf, process::ExitCode};

use makosh_mail_persons_sync_assembly::materialize_mail_persons_sync_assembly_v1;

const OPTIONS: [&str; 3] = ["--build-id", "--output", "--runtime"];

struct Arguments {
    build_id: String,
    output: PathBuf,
    runtime: PathBuf,
}

fn main() -> ExitCode {
    let Some(arguments) = arguments(std::env::args().skip(1).collect()) else {
        return fail(
            "usage: makosh-mail-persons-sync-assembly --build-id <id> \
             --runtime <absolute-path> --output <absolute-path>",
        );
    };
    if materialize_mail_persons_sync_assembly_v1(
        &arguments.output,
        &arguments.build_id,
        &arguments.runtime,
    )
    .is_err()
    {
        return fail("assembly failed");
    }
    println!("mail-persons-sync-release-assembly: ok");
    ExitCode::SUCCESS
}

fn arguments(values: Vec<String>) -> Option<Arguments> {
    if values.len() != OPTIONS.len() * 2 {
        return None;
    }
    let mut parsed = BTreeMap::new();
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
        output: PathBuf::from(parsed.remove("--output")?),
        runtime: PathBuf::from(parsed.remove("--runtime")?),
    })
}

fn fail(message: &str) -> ExitCode {
    eprintln!("mail-persons-sync-release-assembly: {message}");
    ExitCode::FAILURE
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_each_exact_option_once_in_any_order() {
        let parsed = arguments(vec![
            "--runtime".to_owned(),
            "/runtime".to_owned(),
            "--output".to_owned(),
            "/output".to_owned(),
            "--build-id".to_owned(),
            "build-1".to_owned(),
        ])
        .expect("exact arguments");
        assert_eq!(parsed.build_id, "build-1");
        assert_eq!(parsed.runtime, PathBuf::from("/runtime"));
        assert_eq!(parsed.output, PathBuf::from("/output"));
    }

    #[test]
    fn rejects_missing_duplicate_and_unknown_options() {
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
