# Retired legacy OAuth bundle resource

Clean-room Tauri builds never package this directory and never forward OAuth
configuration to `makosh-kernel`. Provider-specific OAuth provisioning belongs
to a future owner/integration boundary after the Vault and Gateway contracts
are implemented.

`client_secret.json` remains ignored so an existing local provisioning file is
not accidentally committed. It is not an allowed Tauri bundle resource or
Kernel environment input.
