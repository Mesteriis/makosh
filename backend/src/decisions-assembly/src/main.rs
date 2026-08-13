use makosh_decisions_assembly::materialize_decisions_release_assembly_v1;
use std::{path::PathBuf, process::ExitCode};

const OPTIONS: [&str; 3] = ["--build-id", "--output-dir", "--runtime"];
struct Arguments {
    build_id: String,
    output_directory: PathBuf,
    runtime: PathBuf,
}

fn main() -> ExitCode {
    let Some(arguments) = arguments(std::env::args().skip(1).collect()) else {
        return fail(
            "usage: makosh-decisions-assembly --build-id <id> --output-dir <absolute-path> --runtime <absolute-path>",
        );
    };
    if materialize_decisions_release_assembly_v1(
        &arguments.output_directory,
        &arguments.build_id,
        &arguments.runtime,
    )
    .is_err()
    {
        return fail("assembly failed");
    }
    println!("decisions-release-assembly: ok");
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
    fn exact_cli_options_are_bounded() {
        let value = arguments(vec![
            "--runtime".into(),
            "/runtime".into(),
            "--build-id".into(),
            "build-1".into(),
            "--output-dir".into(),
            "/output".into(),
        ])
        .expect("arguments");
        assert_eq!(value.build_id, "build-1");
        assert!(arguments(Vec::new()).is_none());
    }
}
