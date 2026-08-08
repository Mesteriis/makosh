#!/usr/bin/env bash
set -euo pipefail

readonly TELEGRAM_IOS_REPOSITORY="https://github.com/TelegramMessenger/Telegram-iOS.git"
readonly TELEGRAM_IOS_COMMIT="6ad963e5b62d354da79040f388ae2b9132fb17b8"
readonly TGCALLS_COMMIT="e3069322a3d1e16ecb11a5e302242e59ddd7f09e"
readonly WEBRTC_COMMIT="3817e906cb6c22ec9cc62023b073e1a668d9cb33"
readonly LIBVPX_COMMIT="e7bfd8b6c230a6824e7fd1efa2378a7322986128"
readonly DAV1D_COMMIT="330e20672e85f9de1678dccd6957845898ef57a1"
readonly TGCALLS_LICENSE_SHA256="da7eabb7bafdf7d3ae5e9f223aa5bdc1eece45ac569dc21b3b037520b4464768"
readonly BAZEL_VERSION="8.4.2"
readonly BAZEL_SHA256="45e9388abf21d1107e146ea366ad080eb93cb6a5f3a4a3b048f78de0bc3faffa"
readonly XCODE_VERSION="26.2"
readonly ARTIFACT_NAME="libmakosh_tgcalls_bridge.dylib"
readonly AUDIO_CONFORMANCE_NAME="makosh_tgcalls_audio_device_conformance"

usage() {
  echo "usage: $0 --output-dir <new-absolute-directory> [--development-audio-conformance]" >&2
}

output_directory=""
build_profile="release"
while (($# > 0)); do
  case "$1" in
    --output-dir)
      (($# >= 2)) || {
        usage
        exit 2
      }
      output_directory="$2"
      shift 2
      ;;
    --development-audio-conformance)
      if [[ "$build_profile" != "release" ]]; then
        usage
        exit 2
      fi
      build_profile="development-audio-conformance"
      shift
      ;;
    *)
      usage
      exit 2
      ;;
  esac
done

if [[ -z "$output_directory" || "$output_directory" != /* || -e "$output_directory" ]]; then
  usage
  exit 2
fi
if [[ "$(uname -s)" != "Darwin" || "$(uname -m)" != "arm64" ]]; then
  echo "the pinned Telegram tgcalls bridge release target requires macOS arm64" >&2
  exit 1
fi
installed_xcode_version="$(xcodebuild -version | sed -n '1s/^Xcode //p')"
if [[ -z "$installed_xcode_version" ]]; then
  echo "the active Xcode version is unavailable" >&2
  exit 1
fi
if [[ "$build_profile" == "release" && "$installed_xcode_version" != "$XCODE_VERSION" ]]; then
  echo "the pinned Telegram-iOS release requires Xcode ${XCODE_VERSION}" >&2
  exit 1
fi

script_directory="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
backend_directory="$(CDPATH= cd -- "${script_directory}/.." && pwd)"
native_directory="${backend_directory}/src/telegram-call-media-tgcalls/native"
patch_path="${native_directory}/patches/telegram-ios-macos-audio-device.patch"
scratch_directory="$(mktemp -d "${TMPDIR:-/tmp}/makosh-tgcalls-build.XXXXXXXX")"
checkout_directory="${scratch_directory}/Telegram-iOS"
release_directory="${scratch_directory}/release"
trap 'rm -rf -- "$scratch_directory"' EXIT

git init -q "$checkout_directory"
git -C "$checkout_directory" remote add origin "$TELEGRAM_IOS_REPOSITORY"
git -C "$checkout_directory" sparse-checkout init --cone
git -C "$checkout_directory" sparse-checkout set \
  build-system \
  third-party \
  submodules/TgVoipWebrtc \
  submodules/ffmpeg
git -C "$checkout_directory" fetch -q --depth 1 origin "$TELEGRAM_IOS_COMMIT"
git -C "$checkout_directory" -c advice.detachedHead=false checkout -q --detach "$TELEGRAM_IOS_COMMIT"

git -C "$checkout_directory" submodule update --init --depth 1 \
  build-system/bazel-rules/apple_support \
  build-system/bazel-rules/rules_apple \
  build-system/bazel-rules/rules_swift \
  build-system/bazel-rules/rules_xcodeproj \
  build-system/bazel-rules/sourcekit-bazel-bsp \
  submodules/TgVoipWebrtc/tgcalls \
  third-party/dav1d/dav1d \
  third-party/libvpx/libvpx \
  third-party/webrtc/webrtc

verify_commit() {
  local relative_path="$1"
  local expected_commit="$2"
  local actual_commit
  actual_commit="$(git -C "${checkout_directory}/${relative_path}" rev-parse HEAD)"
  if [[ "$actual_commit" != "$expected_commit" ]]; then
    echo "unexpected pinned commit for ${relative_path}" >&2
    exit 1
  fi
}

verify_commit "submodules/TgVoipWebrtc/tgcalls" "$TGCALLS_COMMIT"
verify_commit "third-party/webrtc/webrtc" "$WEBRTC_COMMIT"
verify_commit "third-party/libvpx/libvpx" "$LIBVPX_COMMIT"
verify_commit "third-party/dav1d/dav1d" "$DAV1D_COMMIT"

actual_license_sha="$(shasum -a 256 \
  "${checkout_directory}/submodules/TgVoipWebrtc/tgcalls/LICENSE" | awk '{print $1}')"
if [[ "$actual_license_sha" != "$TGCALLS_LICENSE_SHA256" ]]; then
  echo "unexpected tgcalls license bytes" >&2
  exit 1
fi

configuration_directory="${checkout_directory}/build-input/configuration-repository"
mkdir -p "${configuration_directory}/provisioning"
cp "${checkout_directory}/build-system/example-configuration/variables.bzl" \
  "${configuration_directory}/variables.bzl"
touch \
  "${configuration_directory}/WORKSPACE" \
  "${configuration_directory}/BUILD" \
  "${configuration_directory}/provisioning/BUILD"
cat >"${configuration_directory}/MODULE.bazel" <<'EOF'
module(
    name = "build_configuration",
)
EOF

bazel_path="${checkout_directory}/build-input/bazel-${BAZEL_VERSION}-darwin-arm64"
curl --fail --location --silent --show-error \
  "https://github.com/bazelbuild/bazel/releases/download/${BAZEL_VERSION}/bazel-${BAZEL_VERSION}-darwin-arm64" \
  --output "$bazel_path"
actual_bazel_sha="$(shasum -a 256 "$bazel_path" | awk '{print $1}')"
if [[ "$actual_bazel_sha" != "$BAZEL_SHA256" ]]; then
  echo "unexpected Bazel bytes" >&2
  exit 1
fi
chmod 0555 "$bazel_path"

bridge_directory="${checkout_directory}/makosh-tgcalls-bridge"
mkdir "$bridge_directory"
cp \
  "${native_directory}/BUILD.bazel" \
  "${native_directory}/audio_device_conformance.cpp" \
  "${native_directory}/bridge.cpp" \
  "${native_directory}/bridge.h" \
  "$bridge_directory/"
git -C "$checkout_directory" apply --check "$patch_path"
git -C "$checkout_directory" apply "$patch_path"

(
  cd "$checkout_directory"
  build_targets=(
    //makosh-tgcalls-bridge:libmakosh_tgcalls_bridge.dylib
  )
  if [[ "$build_profile" == "development-audio-conformance" ]]; then
    build_targets+=(
      //makosh-tgcalls-bridge:makosh_tgcalls_audio_device_conformance
    )
  fi
  "$bazel_path" build \
    "${build_targets[@]}" \
    -c opt \
    --stamp=false
)

mkdir "$release_directory"
install -m 0555 \
  "${checkout_directory}/bazel-bin/makosh-tgcalls-bridge/${ARTIFACT_NAME}" \
  "${release_directory}/${ARTIFACT_NAME}"
install -m 0444 \
  "${checkout_directory}/submodules/TgVoipWebrtc/tgcalls/LICENSE" \
  "${release_directory}/LICENSE.tgcalls-LGPL-3.0"
release_eligible=true
audio_conformance_artifact=null
if [[ "$build_profile" == "development-audio-conformance" ]]; then
  install -m 0555 \
    "${checkout_directory}/bazel-bin/makosh-tgcalls-bridge/${AUDIO_CONFORMANCE_NAME}" \
    "${release_directory}/${AUDIO_CONFORMANCE_NAME}"
  release_eligible=false
  audio_conformance_artifact="\"${AUDIO_CONFORMANCE_NAME}\""
fi

artifact_sha="$(shasum -a 256 "${release_directory}/${ARTIFACT_NAME}" | awk '{print $1}')"
patch_sha="$(shasum -a 256 "$patch_path" | awk '{print $1}')"
cat >"${release_directory}/provenance.json" <<EOF
{
  "artifact": "${ARTIFACT_NAME}",
  "artifact_sha256": "${artifact_sha}",
  "audio_device_conformance_artifact": ${audio_conformance_artifact},
  "bazel_sha256": "${BAZEL_SHA256}",
  "bazel_version": "${BAZEL_VERSION}",
  "bridge_abi": 1,
  "build_profile": "${build_profile}",
  "build_target": "//makosh-tgcalls-bridge:libmakosh_tgcalls_bridge.dylib",
  "dav1d_commit": "${DAV1D_COMMIT}",
  "libvpx_commit": "${LIBVPX_COMMIT}",
  "patch_sha256": "${patch_sha}",
  "platform": "darwin-arm64",
  "release_eligible": ${release_eligible},
  "telegram_ios_commit": "${TELEGRAM_IOS_COMMIT}",
  "tgcalls_commit": "${TGCALLS_COMMIT}",
  "tgcalls_license": "LGPL-3.0",
  "tgcalls_license_sha256": "${TGCALLS_LICENSE_SHA256}",
  "webrtc_commit": "${WEBRTC_COMMIT}",
  "xcode_version": "${installed_xcode_version}",
  "xcode_version_pin": "${XCODE_VERSION}"
}
EOF
chmod 0444 "${release_directory}/provenance.json"

mv "$release_directory" "$output_directory"
trap - EXIT
rm -rf -- "$scratch_directory"
echo "built ${output_directory}/${ARTIFACT_NAME}"
