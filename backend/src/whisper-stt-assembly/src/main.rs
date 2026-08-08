use std::{ffi::OsStr, path::PathBuf};

use makosh_whisper_stt_assembly::materialize_whisper_stt_release_assembly_v1;

fn main() -> Result<(), String> {
    let mut arguments = std::env::args_os();
    let _binary = arguments.next();
    let output = required(&mut arguments, "--output")?;
    let build_id = required(&mut arguments, "--build-id")?
        .into_os_string()
        .into_string()
        .map_err(|_| "Whisper STT build id is invalid".to_owned())?;
    let runtime = required(&mut arguments, "--runtime")?;
    let runner = required(&mut arguments, "--runner")?;
    let model = required(&mut arguments, "--model")?;
    if arguments.next().is_some() {
        return Err("Whisper STT assembly arguments are invalid".to_owned());
    }
    materialize_whisper_stt_release_assembly_v1(&output, &build_id, &runtime, &runner, &model)
        .map(|_| ())
        .map_err(|error| format!("Whisper STT assembly failed: {error:?}"))
}

fn required(
    arguments: &mut impl Iterator<Item = std::ffi::OsString>,
    flag: &str,
) -> Result<PathBuf, String> {
    if arguments.next().as_deref() != Some(OsStr::new(flag)) {
        return Err("Whisper STT assembly arguments are invalid".to_owned());
    }
    arguments
        .next()
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .ok_or_else(|| "Whisper STT assembly arguments are invalid".to_owned())
}
