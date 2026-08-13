use makosh_search_assembly::materialize_search_release_assembly_v1;
use std::{path::PathBuf, process::ExitCode};

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() != 6 {
        return fail();
    }
    let mut values = std::collections::BTreeMap::new();
    for pair in args.chunks_exact(2) {
        if !["--build-id", "--output-dir", "--runtime"].contains(&pair[0].as_str())
            || pair[1].is_empty()
            || values.insert(pair[0].clone(), pair[1].clone()).is_some()
        {
            return fail();
        }
    }
    let (Some(build), Some(output), Some(runtime)) = (
        values.remove("--build-id"),
        values.remove("--output-dir"),
        values.remove("--runtime"),
    ) else {
        return fail();
    };
    if materialize_search_release_assembly_v1(
        &PathBuf::from(output),
        &build,
        &PathBuf::from(runtime),
    )
    .is_err()
    {
        return fail();
    }
    println!("search-release-assembly: ok");
    ExitCode::SUCCESS
}
fn fail() -> ExitCode {
    eprintln!(
        "usage: makosh-search-assembly --build-id <id> --output-dir <absolute-path> --runtime <absolute-path>"
    );
    ExitCode::FAILURE
}
