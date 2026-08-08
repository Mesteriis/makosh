#!/usr/bin/env bash
set -euo pipefail

# Release-only supply unit. Runtime never downloads or discovers provider bytes.
export PATH="/usr/bin:/bin:/usr/sbin:/sbin"
readonly PATH

readonly WHISPER_REPOSITORY="https://github.com/ggml-org/whisper.cpp.git"
readonly WHISPER_COMMIT="23ee03506a91ac3d3f0071b40e66a430eebdfa1d"
readonly WHISPER_LICENSE_SHA256="94f29bbed6a22c35b992c5c6ebf0e7c92f13b836b90f36f461c9cf2f0f1d010d"
readonly MODEL_REPOSITORY_REVISION="5359861c739e955e79d9a303bcbc70fb988958b1"
readonly MODEL_URL="https://huggingface.co/ggerganov/whisper.cpp/resolve/${MODEL_REPOSITORY_REVISION}/ggml-base.bin"
readonly MODEL_SHA256="60ed5bc3dd14eea856493d334349b405782ddcaf0028d4b5df4088345fba2efe"
readonly MODEL_SIZE_BYTES="147951465"
readonly MODEL_README_URL="https://huggingface.co/ggerganov/whisper.cpp/raw/${MODEL_REPOSITORY_REVISION}/README.md"
readonly MODEL_README_SHA256="21fd967098804f33fc84e803fb0e5ab7666d71801f4027cf28a65e7af09c1758"
readonly CMAKE_VERSION="4.4.2"
readonly CMAKE_ARCHIVE_SHA256="800fc86838e913fff969b499886c80baeb4ccfd00f0e39906b34aa334f39ab6c"
readonly XCODE_VERSION="26.6"
readonly MACOS_SDK_VERSION="26.5"
readonly MACOS_SDK_BUILD="25F70"
readonly CLANG_SHA256="7def90dd8829726686213a747fc5bff1583df933dae5edc55d755479e0bfe00a"
readonly LINKER_SHA256="5897b275efd93b201b6df5832dd541262b3f20f290859ba78f2200a6a66ef38b"
readonly DEPLOYMENT_TARGET="13.0"
readonly RUNNER_NAME="whisper-cli"
readonly MODEL_NAME="ggml-base.bin"

usage() {
  echo "usage: $0 --output-dir <new-absolute-directory> [--verify-reproducibility]" >&2
}

output_directory=""
verify_reproducibility=false
while (($# > 0)); do
  case "$1" in
    --output-dir)
      (($# >= 2)) || { usage; exit 2; }
      output_directory="$2"
      shift 2
      ;;
    --verify-reproducibility)
      [[ "$verify_reproducibility" == false ]] || { usage; exit 2; }
      verify_reproducibility=true
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
  echo "the pinned Whisper STT release target requires macOS arm64" >&2
  exit 1
fi

installed_xcode_version="$(xcodebuild -version | sed -n '1s/^Xcode //p')"
installed_sdk_version="$(xcrun --sdk macosx --show-sdk-version)"
installed_sdk_build="$(xcrun --sdk macosx --show-sdk-build-version)"
clang_path="$(xcrun --find clang)"
clangxx_path="$(xcrun --find clang++)"
linker_path="$(xcrun --find ld)"
sdk_path="$(xcrun --sdk macosx --show-sdk-path)"
if [[ "$installed_xcode_version" != "$XCODE_VERSION" \
  || "$installed_sdk_version" != "$MACOS_SDK_VERSION" \
  || "$installed_sdk_build" != "$MACOS_SDK_BUILD" \
  || "$(shasum -a 256 "$clang_path" | cut -d ' ' -f 1)" != "$CLANG_SHA256" \
  || "$(shasum -a 256 "$clangxx_path" | cut -d ' ' -f 1)" != "$CLANG_SHA256" \
  || "$(shasum -a 256 "$linker_path" | cut -d ' ' -f 1)" != "$LINKER_SHA256" ]]; then
  echo "the active Xcode toolchain does not match the pinned Whisper STT release toolchain" >&2
  exit 1
fi

scratch_directory="$(mktemp -d "${TMPDIR:-/tmp}/makosh-whisper-build.XXXXXXXX")"
trap 'rm -rf -- "$scratch_directory"' EXIT
source_directory="${scratch_directory}/whisper.cpp"
git init -q "$source_directory"
git -C "$source_directory" remote add origin "$WHISPER_REPOSITORY"
git -C "$source_directory" fetch -q --depth 1 origin "$WHISPER_COMMIT"
git -C "$source_directory" -c advice.detachedHead=false checkout -q --detach "$WHISPER_COMMIT"
if [[ "$(git -C "$source_directory" rev-parse HEAD)" != "$WHISPER_COMMIT" ]]; then
  echo "the Whisper STT source checkout does not match its pinned commit" >&2
  exit 1
fi

verify_sha256() {
  local path="$1"
  local expected="$2"
  local label="$3"
  if [[ ! -f "$path" || -L "$path" \
    || "$(shasum -a 256 "$path" | cut -d ' ' -f 1)" != "$expected" ]]; then
    echo "unexpected pinned ${label} bytes" >&2
    exit 1
  fi
}

verify_sha256 "${source_directory}/LICENSE" "$WHISPER_LICENSE_SHA256" "whisper.cpp license"
model_path="${scratch_directory}/${MODEL_NAME}"
model_readme_path="${scratch_directory}/MODEL-README.md"
curl --fail --location --silent --show-error "$MODEL_URL" --output "$model_path"
curl --fail --location --silent --show-error "$MODEL_README_URL" --output "$model_readme_path"
verify_sha256 "$model_path" "$MODEL_SHA256" "Whisper model"
verify_sha256 "$model_readme_path" "$MODEL_README_SHA256" "Whisper model README"
if [[ "$(stat -f '%z' "$model_path")" != "$MODEL_SIZE_BYTES" ]]; then
  echo "unexpected pinned Whisper model size" >&2
  exit 1
fi

cmake_archive="${scratch_directory}/cmake-${CMAKE_VERSION}-macos-universal.tar.gz"
curl --fail --location --silent --show-error \
  "https://github.com/Kitware/CMake/releases/download/v${CMAKE_VERSION}/cmake-${CMAKE_VERSION}-macos-universal.tar.gz" \
  --output "$cmake_archive"
verify_sha256 "$cmake_archive" "$CMAKE_ARCHIVE_SHA256" "CMake archive"
tar -xzf "$cmake_archive" -C "$scratch_directory"
cmake_path="${scratch_directory}/cmake-${CMAKE_VERSION}-macos-universal/CMake.app/Contents/bin/cmake"
if [[ ! -x "$cmake_path" || "$($cmake_path --version | sed -n '1s/^cmake version //p')" != "$CMAKE_VERSION" ]]; then
  echo "the pinned CMake executable is unavailable" >&2
  exit 1
fi

source_date_epoch="$(git -C "$source_directory" show -s --format=%ct HEAD)"
readonly source_date_epoch

build_once() {
  local build_root="$1"
  local release_root="$2"
  local mapped_flags="-O2 -DNDEBUG -isysroot ${sdk_path} -ffile-prefix-map=${source_directory}=/usr/src/makosh-whisper/source -fdebug-prefix-map=${source_directory}=/usr/src/makosh-whisper/source -ffile-prefix-map=${build_root}=/usr/src/makosh-whisper/build -fdebug-prefix-map=${build_root}=/usr/src/makosh-whisper/build"
  export SOURCE_DATE_EPOCH="$source_date_epoch"
  export ZERO_AR_DATE=1
  export LC_ALL=C
  export LANG=C
  export TZ=UTC
  export SDKROOT="$sdk_path"
  "$cmake_path" -S "$source_directory" -B "$build_root" \
    -G "Unix Makefiles" \
    -DCMAKE_MAKE_PROGRAM=/usr/bin/make \
    -DCMAKE_BUILD_TYPE=Release \
    -DCMAKE_OSX_ARCHITECTURES=arm64 \
    -DCMAKE_OSX_DEPLOYMENT_TARGET="$DEPLOYMENT_TARGET" \
    -DCMAKE_OSX_SYSROOT="$sdk_path" \
    -DCMAKE_C_COMPILER="$clang_path" \
    -DCMAKE_CXX_COMPILER="$clangxx_path" \
    -DCMAKE_C_FLAGS_RELEASE="$mapped_flags" \
    -DCMAKE_CXX_FLAGS_RELEASE="${mapped_flags} -stdlib=libc++" \
    -DCMAKE_FIND_USE_PACKAGE_REGISTRY=OFF \
    -DCMAKE_FIND_USE_SYSTEM_PACKAGE_REGISTRY=OFF \
    -DCMAKE_FIND_USE_SYSTEM_ENVIRONMENT_PATH=OFF \
    -DCMAKE_IGNORE_PATH="/opt/homebrew/bin;/opt/homebrew/sbin;/usr/local/bin;/usr/local/sbin" \
    -DCMAKE_IGNORE_PREFIX_PATH="/opt/homebrew;/usr/local" \
    -DBUILD_SHARED_LIBS=OFF \
    -DGGML_STATIC=ON \
    -DGGML_NATIVE=OFF \
    -DGGML_CCACHE=OFF \
    -DGGML_BLAS=OFF \
    -DGGML_METAL=OFF \
    -DGGML_OPENMP=OFF \
    -DGGML_RPC=OFF \
    -DWHISPER_COREML=OFF \
    -DWHISPER_CURL=OFF \
    -DWHISPER_BUILD_TESTS=OFF \
    -DWHISPER_BUILD_SERVER=OFF \
    -DWHISPER_BUILD_EXAMPLES=ON
  "$cmake_path" --build "$build_root" --target whisper-cli --parallel
  mkdir "$release_root"
  install -m 0555 "${build_root}/bin/whisper-cli" "${release_root}/${RUNNER_NAME}"
  install -m 0444 "$model_path" "${release_root}/${MODEL_NAME}"
  install -m 0444 "${source_directory}/LICENSE" "${release_root}/LICENSE.whisper.cpp-MIT"
  install -m 0444 "$model_readme_path" "${release_root}/MODEL-README.md"
  local non_system_loads
  non_system_loads="$(otool -L "${release_root}/${RUNNER_NAME}" | tail -n +2 | awk '{print $1}' \
    | grep -Ev '^(/usr/lib/|/System/Library/)' || true)"
  if [[ -n "$non_system_loads" ]] || otool -l "${release_root}/${RUNNER_NAME}" | grep -q 'LC_RPATH'; then
    echo "the Whisper STT runner has an unapproved dynamic dependency" >&2
    exit 1
  fi
  {
    echo "${RUNNER_NAME}:"
    otool -L "${release_root}/${RUNNER_NAME}" | tail -n +2
  } >"${release_root}/dynamic-dependencies.txt"
  chmod 0444 "${release_root}/dynamic-dependencies.txt"
}

first_build="${scratch_directory}/build-first"
first_release="${scratch_directory}/release-first"
build_once "$first_build" "$first_release"
runner_sha="$(shasum -a 256 "${first_release}/${RUNNER_NAME}" | cut -d ' ' -f 1)"
reproducibility_verified=false
if [[ "$verify_reproducibility" == true ]]; then
  second_build="${scratch_directory}/build-second"
  second_release="${scratch_directory}/release-second"
  build_once "$second_build" "$second_release"
  for artifact in "$RUNNER_NAME" "$MODEL_NAME" dynamic-dependencies.txt; do
    cmp -s "${first_release}/${artifact}" "${second_release}/${artifact}" || {
      echo "the isolated Whisper STT builds are not reproducible" >&2
      exit 1
    }
  done
  reproducibility_verified=true
fi

cat >"${first_release}/provenance.json" <<EOF
{
  "artifact": "${RUNNER_NAME}",
  "artifact_sha256": "${runner_sha}",
  "cmake_archive_sha256": "${CMAKE_ARCHIVE_SHA256}",
  "model_revision": "${MODEL_REPOSITORY_REVISION}",
  "model_sha256": "${MODEL_SHA256}",
  "release_eligible": ${reproducibility_verified},
  "target": "aarch64-apple-darwin",
  "whisper_commit": "${WHISPER_COMMIT}"
}
EOF
chmod 0444 "${first_release}/provenance.json"
mv "$first_release" "$output_directory"
trap - EXIT
rm -rf -- "$scratch_directory"
