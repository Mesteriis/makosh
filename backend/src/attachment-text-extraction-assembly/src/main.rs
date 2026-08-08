use std::{collections::BTreeMap, path::PathBuf, process::ExitCode};

use makosh_attachment_text_extraction_assembly::materialize_attachment_text_extraction_release_assembly_v1;

const OPTIONS: [&str; 6] = [
    "--build-id",
    "--ocr-eng",
    "--ocr-runner",
    "--ocr-rus",
    "--output-dir",
    "--runtime",
];

fn main() -> ExitCode {
    let Some(arguments) = arguments(std::env::args().skip(1).collect()) else {
        return fail(
            "usage: makosh-attachment-text-extraction-assembly --build-id <id> \
             --output-dir <absolute-path> --runtime <absolute-path> \
             --ocr-runner <absolute-path> --ocr-eng <absolute-path> \
             --ocr-rus <absolute-path>",
        );
    };
    if materialize_attachment_text_extraction_release_assembly_v1(
        &arguments.output_directory,
        &arguments.build_id,
        &arguments.runtime,
        &arguments.ocr_runner,
        &arguments.ocr_english,
        &arguments.ocr_russian,
    )
    .is_err()
    {
        return fail("assembly failed");
    }
    println!("attachment-text-extraction-release-assembly: ok");
    ExitCode::SUCCESS
}

struct Arguments {
    build_id: String,
    output_directory: PathBuf,
    runtime: PathBuf,
    ocr_runner: PathBuf,
    ocr_english: PathBuf,
    ocr_russian: PathBuf,
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
        ocr_runner: PathBuf::from(parsed.remove("--ocr-runner")?),
        ocr_english: PathBuf::from(parsed.remove("--ocr-eng")?),
        ocr_russian: PathBuf::from(parsed.remove("--ocr-rus")?),
    })
}

fn fail(message: &str) -> ExitCode {
    eprintln!("attachment-text-extraction-release-assembly: {message}");
    ExitCode::FAILURE
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_each_exact_option_once() {
        assert!(
            arguments(vec![
                "--runtime".to_owned(),
                "/tmp/runtime".to_owned(),
                "--ocr-runner".to_owned(),
                "/tmp/tesseract-runner".to_owned(),
                "--ocr-eng".to_owned(),
                "/tmp/eng.traineddata".to_owned(),
                "--ocr-rus".to_owned(),
                "/tmp/rus.traineddata".to_owned(),
                "--build-id".to_owned(),
                "build-1".to_owned(),
                "--output-dir".to_owned(),
                "/tmp/output".to_owned(),
            ])
            .is_some()
        );
        assert!(arguments(Vec::new()).is_none());
    }
}
