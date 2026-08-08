# Telegram tgcalls media adapter

This integration-owned build unit contains the Rust loader/session adapter and
the narrow native C ABI used by Telegram one-to-one audio calls. It is not a
domain, assembly, independently managed runtime, or generic media service.

The release artifact is built from exact Telegram-iOS, tgcalls, WebRTC, Bazel
and Xcode inputs:

```sh
backend/scripts/build-telegram-tgcalls-bridge-macos.sh \
  --output-dir /absolute/new/output-directory
```

The command refuses an existing output directory, a non-arm64 macOS host, an
Xcode version other than the version pinned by Telegram-iOS, altered upstream
commits, altered tgcalls license bytes, or altered Bazel bytes. Its output
contains the dylib, the upstream LGPL-3.0 license and a provenance manifest.

For development-only CoreAudio conformance, the same exact source, dependency,
license and Bazel pins can build a separate test binary on the active Xcode:

```sh
backend/scripts/build-telegram-tgcalls-bridge-macos.sh \
  --output-dir /absolute/new/development-output-directory \
  --development-audio-conformance
```

This profile records `release_eligible: false`, the active Xcode version and
the required release Xcode pin in `provenance.json`. Its dylib must not enter a
signed release. The extra
`makosh_tgcalls_audio_device_conformance` binary is a test build unit and is not
referenced by Telegram assembly.

Running the binary accesses the default microphone and speaker and therefore
requires an exact explicit flag:

```sh
/absolute/development-output-directory/makosh_tgcalls_audio_device_conformance \
  --allow-microphone-and-speaker-access
```

The probe discards captured samples, supplies only silence to playout, persists
no audio, restores the original microphone mute state and exits after a bounded
full-duplex callback threshold. Building it does not access audio devices.

The pinned Telegram-iOS `tgcalls_core` target omits the macOS implementation of
`AudioDeviceModule::Create` from final consumers. The exact source patch in
`native/patches/` adds only that CoreAudio build unit; the production bridge
still uses the platform-default input and output devices. Fake PCM adapters are
not linked into the production bridge.

This build and its loader conformance do not by themselves open
`telegram_call_media_v1`. An exact Xcode-pinned release artifact, native
ownership/callback/shutdown-failure conformance, an explicitly run real
audio-device check and an authorized live one-to-one call remain separate
admission evidence under ADR-0284.
