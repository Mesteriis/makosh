# Tauri Sidecars

This directory holds generated Tauri sidecar binaries.

Apple Silicon release builds expect the signed Kernel artifact named according
to the Tauri external-binary contract:

- `makosh-kernel-aarch64-apple-darwin`

The desktop host starts it only as `makosh-kernel serve` after clearing its
inherited environment. It passes no API secret, provider credential, database
address or other environment-derived configuration to the child. The host can
perform at most three bounded restart attempts and never supervises Kernel
children.

Generate the sidecar from the clean-room backend with:

```sh
make -C backend package-kernel-macos \
  RELEASE_TRUST_ROOT=/absolute/makosh-release-trust-root.pb \
  SIGNED_DISTRIBUTION_MANIFEST=/absolute/makosh-signed-distribution-manifest.pb \
  DISTRIBUTION_BUNDLE=/absolute/makosh-distribution
```

`RELEASE_TRUST_ROOT` is a release-pipeline-produced `ReleaseTrustRootV1` that
contains only public distribution verification keys. Its matching private
signing material never enters the app bundle or repository. The signed
distribution manifest plus exact staged bytes are the required trust baseline;
Apple code signing and notarization are optional independent hardening.
Generated binaries and copied release resources are local build artifacts and
are not committed.
