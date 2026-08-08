# macOS Tauri release manifest

The file-backed P-256 `ReleaseTrustRootV1` and raw-byte-signed
`DistributionManifestV1` are the managed-launch trust baseline. They can be
packaged without an Apple Developer ID or notarization host.

## Local signed package (current baseline)

The current supported desktop delivery is a local `.app` whose trust root and
managed distribution are bound by the P-256 release authority. Its Kernel
sidecar and app bundle are covered by an ad-hoc macOS code signature. This is a
**local signed distribution**, not an Apple-notarized or publicly trusted macOS
release: Tauri first builds without an Apple identity, then
`package-local-macos` applies and verifies that local signature. The owner may
still need to approve the app locally before first launch.

Generate the release authority once, outside the repository, prepare the
canonical distribution input, then build and package it:

```sh
make -C backend generate-release-signing-key \
  RELEASE_SIGNING_KEY=/absolute/release-p256.pem

make -C backend build-distribution-release \
  RELEASE_DISTRIBUTION_INPUT=/absolute/distribution-release.json \
  RELEASE_SIGNING_KEY=/absolute/release-p256.pem \
  RELEASE_TRUST_ROOT=/absolute/makosh-release-trust-root.pb \
  SIGNED_DISTRIBUTION_MANIFEST=/absolute/makosh-signed-distribution-manifest.pb \
  DISTRIBUTION_BUNDLE=/absolute/makosh-distribution

make -C backend package-local-macos \
  RELEASE_TRUST_ROOT=/absolute/makosh-release-trust-root.pb \
  SIGNED_DISTRIBUTION_MANIFEST=/absolute/makosh-signed-distribution-manifest.pb \
  DISTRIBUTION_BUNDLE=/absolute/makosh-distribution
```

`package-local-macos` first stages the exact signed resources and Kernel
sidecar, then runs the frontend build and Tauri app bundle with no Apple
identity. It adds an ad-hoc code signature only after Tauri has assembled the
bundle; this is local code integrity, not a Developer-ID certificate. Tauri
consumes the target-qualified staged input and packages the runtime sidecar as
`Contents/MacOS/makosh-kernel`, which is also the exact path the Kernel loader
and optional Apple-release verifier require. Keep the P-256 private key
owner-readable and outside Git. Its public
verification key is embedded into `ReleaseTrustRootV1`; key rotation remains
the existing explicit `additional_verification_keys` procedure.

Do not run `verify-macos-release` for this local baseline: that verifier is
specifically for a future Apple Developer-ID-signed, notarized bundle.

`verify-macos-release-manifest.mjs` is optional release hardening for a signed
Tauri application bundle. When used, it checks the packaged
and the packaged `makosh-kernel-aarch64-apple-darwin` sidecar on an Apple
Silicon macOS release host. The manifest requires the expected Apple Team ID,
the exact SHA-256 digest of the sidecar and the exact SHA-256 digest of the
binary `ReleaseTrustRootV1` resource, signed `DistributionManifestV1` envelope
and managed distribution bundle at their exact paths inside the signed `.app`
bundle. The tool rejects symlinks before hashing, then runs
`codesign --verify --strict`, verifies the team identity, requires a stapled
notarization ticket with `xcrun stapler validate`, and runs Gatekeeper
assessment for the app bundle.

It has no software-signing fallback: the command either proves the Apple
signing/notarization ceremony or fails. This is an optional future upgrade, not
a requirement for the local signed baseline. Unit tests validate the manifest
shape but do not claim notarization; an operator with a release host validates
it with:

```sh
make -C backend verify-macos-release MANIFEST=/absolute/macos-release.json
```

Before Tauri packaging, the release pipeline can produce the inner binary
artifacts with:

```sh
make -C backend build-distribution-release \
  RELEASE_DISTRIBUTION_INPUT=/absolute/distribution-release.json \
  RELEASE_SIGNING_KEY=/absolute/release-p256.pem \
  RELEASE_TRUST_ROOT=/absolute/makosh-release-trust-root.pb \
  SIGNED_DISTRIBUTION_MANIFEST=/absolute/makosh-signed-distribution-manifest.pb \
  DISTRIBUTION_BUNDLE=/absolute/makosh-distribution
```

For a new offline/local release authority, generate the P-256 signing key once
at an explicitly chosen external path:

```sh
make -C backend generate-release-signing-key \
  RELEASE_SIGNING_KEY=/absolute/release-p256.pem
```

The command refuses overwrite and emits no key material. Keep that file outside
the repository with owner-only filesystem access.

The compiler materializes every digest-verified declared artifact at its
bundle-relative name below DISTRIBUTION_BUNDLE; the directory must not exist
before the command. It verifies the copied bytes again before atomically
publishing the directory, so package-kernel-macos receives exactly the artifact
inventory that the signed manifest describes, including the required browser
bootstrap document.

For a browser release, first build a reachable, fixed asset inventory rather
than passing all of Vite `dist/` to the release compiler:

```sh
cd frontend
pnpm exec vite build
node scripts/prepare-signed-browser-assets.mjs dist /absolute/browser-release
```

Use `/absolute/browser-release/index.html` as `browser_bootstrap` and
`/absolute/browser-release/assets` as `browser_assets_dir` when generating the
local release input. The preparer follows only exact Vite HTML/JS/CSS
references, rejects symlinks and missing files, and enforces browser inventory
limits before the manifest is signed. Gateway serves no directory fallback.

The JSON lists only canonical artifact paths and their declared bundle-relative
names, plus `additional_verification_keys`: a strictly ordered list of
`{ "key_id", "public_key_path" }` P-256 public PEM files for an explicit key
rotation. The active `verification_key_id` is signed by `RELEASE_SIGNING_KEY`;
the compiler derives that public key and emits it together with the additional
keys in a deterministic `ReleaseTrustRootV1`. Private material is rejected in
`public_key_path`. The compiler calculates every artifact size/digest itself,
writes a bounded binary `DistributionManifestV1` and signs its raw bytes with
P-256. The file key is release-pipeline-only: it is not a device signer, Vault
wrapping key, Tauri resource or Kernel input. The compiler rejects
group/other-readable signing keys, symlink traversal, unordered artifact IDs
and output overwrite.

`package-kernel-macos` needs only the file-backed trust root, signed manifest
and distribution bundle. Apple signing identity is intentionally not compiled
into Kernel; Tauri never forwards it to the sidecar.
