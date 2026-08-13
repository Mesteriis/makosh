use makosh_persons_assembly::materialize_persons_release_assembly_v1;
use std::path::PathBuf;

fn main() -> Result<(), String> {
    let mut args = std::env::args_os().skip(1);
    if args.next().as_deref() != Some(std::ffi::OsStr::new("--build-id")) {
        return Err(
            "usage: makosh-persons-assembly --build-id <id> --runtime <path> --output <dir>"
                .to_owned(),
        );
    }
    let build_id = args
        .next()
        .and_then(|v| v.into_string().ok())
        .ok_or_else(|| "build id required".to_owned())?;
    if args.next().as_deref() != Some(std::ffi::OsStr::new("--runtime")) {
        return Err("runtime required".to_owned());
    }
    let runtime = args
        .next()
        .map(PathBuf::from)
        .ok_or_else(|| "runtime required".to_owned())?;
    if args.next().as_deref() != Some(std::ffi::OsStr::new("--output")) {
        return Err("output required".to_owned());
    }
    let output = args
        .next()
        .map(PathBuf::from)
        .ok_or_else(|| "output required".to_owned())?;
    if args.next().is_some() {
        return Err("unexpected argument".to_owned());
    }
    materialize_persons_release_assembly_v1(&output, &build_id, &runtime)
        .map_err(|_| "persons assembly failed".to_owned())?;
    println!("persons-release-assembly: ok");
    Ok(())
}
