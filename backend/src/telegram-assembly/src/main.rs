use std::collections::BTreeMap;
use std::path::PathBuf;
use std::process::ExitCode;

use makosh_telegram_assembly::materialize_telegram_release_assembly_v1;

const OPTIONS: [&str; 5] = [
    "--build-id",
    "--output-dir",
    "--runtime",
    "--tdjson",
    "--tgcalls",
];

fn main() -> ExitCode {
    match arguments(std::env::args().skip(1).collect()) {
        Some(arguments) => {
            if materialize_telegram_release_assembly_v1(
                &arguments.output_directory,
                &arguments.build_id,
                &arguments.runtime,
                &arguments.tdjson,
                &arguments.tgcalls,
            )
            .is_err()
            {
                return fail("assembly failed");
            }
            println!("telegram-release-assembly: ok");
            ExitCode::SUCCESS
        }
        None => fail(
            "usage: makosh-telegram-assembly --build-id <id> --output-dir <absolute-path> \
             --runtime <absolute-path> --tdjson <absolute-path> --tgcalls <absolute-path>",
        ),
    }
}

struct Arguments {
    build_id: String,
    output_directory: PathBuf,
    runtime: PathBuf,
    tdjson: PathBuf,
    tgcalls: PathBuf,
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
        output_directory: PathBuf::from(parsed.remove("--output-dir")?),
        runtime: PathBuf::from(parsed.remove("--runtime")?),
        tdjson: PathBuf::from(parsed.remove("--tdjson")?),
        tgcalls: PathBuf::from(parsed.remove("--tgcalls")?),
    })
}

fn fail(message: &str) -> ExitCode {
    eprintln!("telegram-release-assembly: {message}");
    ExitCode::FAILURE
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_each_exact_option_once_in_any_order() {
        let parsed = arguments(vec![
            "--tgcalls".to_owned(),
            "/tmp/libmakosh_tgcalls_bridge.dylib".to_owned(),
            "--tdjson".to_owned(),
            "/tmp/libtdjson.dylib".to_owned(),
            "--build-id".to_owned(),
            "build-1".to_owned(),
            "--runtime".to_owned(),
            "/tmp/makosh-telegram-runtime".to_owned(),
            "--output-dir".to_owned(),
            "/tmp/telegram-assembly".to_owned(),
        ])
        .expect("exact arguments");

        assert_eq!(parsed.build_id, "build-1");
        assert_eq!(
            parsed.output_directory,
            PathBuf::from("/tmp/telegram-assembly")
        );
        assert_eq!(
            parsed.runtime,
            PathBuf::from("/tmp/makosh-telegram-runtime")
        );
        assert_eq!(parsed.tdjson, PathBuf::from("/tmp/libtdjson.dylib"));
        assert_eq!(
            parsed.tgcalls,
            PathBuf::from("/tmp/libmakosh_tgcalls_bridge.dylib")
        );
    }

    #[test]
    fn rejects_unknown_duplicate_and_missing_options() {
        assert!(arguments(vec![]).is_none());
        assert!(
            arguments(vec![
                "--build-id".to_owned(),
                "build-1".to_owned(),
                "--build-id".to_owned(),
                "build-2".to_owned(),
                "--runtime".to_owned(),
                "/tmp/runtime".to_owned(),
                "--tdjson".to_owned(),
                "/tmp/tdjson".to_owned(),
                "--tgcalls".to_owned(),
                "/tmp/tgcalls".to_owned(),
            ])
            .is_none()
        );
        assert!(
            arguments(vec![
                "--build-id".to_owned(),
                "build-1".to_owned(),
                "--output-dir".to_owned(),
                "/tmp/output".to_owned(),
                "--runtime".to_owned(),
                "/tmp/runtime".to_owned(),
                "--unknown".to_owned(),
                "/tmp/tdjson".to_owned(),
                "--tgcalls".to_owned(),
                "/tmp/tgcalls".to_owned(),
            ])
            .is_none()
        );
    }
}
