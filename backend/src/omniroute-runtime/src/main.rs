use makosh_omniroute_api::omniroute_settings_schema_bytes_v1;
use makosh_omniroute_persistence::omniroute_storage_bundle_v1;
use makosh_omniroute_runtime::omniroute_module_descriptor_v1;
use prost::Message;
use std::ffi::OsStr;
fn main() -> Result<(), String> {
    let mut args = std::env::args_os();
    let _ = args.next();
    match args.next().as_deref() {
        Some(v) if v == OsStr::new("export-storage-bundle") => {
            write(omniroute_storage_bundle_v1().encode_to_vec(), args.next())
        }
        Some(v) if v == OsStr::new("export-settings-schema") => {
            write(omniroute_settings_schema_bytes_v1(), args.next())
        }
        Some(v) if v == OsStr::new("export-module-descriptor") => {
            let build = args
                .next()
                .and_then(|v| v.into_string().ok())
                .filter(|v| !v.is_empty())
                .ok_or_else(|| "OmniRoute descriptor build id is required".to_owned())?;
            write(
                omniroute_module_descriptor_v1(&build).encode_to_vec(),
                args.next(),
            )
        }
        Some(v) if v == OsStr::new("serve-inherited") => {
            Err("OmniRoute provider credential is unavailable".into())
        }
        _ => Err("OmniRoute runtime command is unavailable".into()),
    }
}
fn write(bytes: Vec<u8>, extra: Option<std::ffi::OsString>) -> Result<(), String> {
    if extra.is_some() {
        return Err("OmniRoute runtime arguments are invalid".into());
    }
    std::io::Write::write_all(&mut std::io::stdout(), &bytes)
        .map_err(|_| "OmniRoute artifact is unavailable".into())
}
