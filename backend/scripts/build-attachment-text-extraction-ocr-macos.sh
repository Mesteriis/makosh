#!/usr/bin/env bash
set -euo pipefail

# The native release build must not discover package-manager tools or libraries
# from the developer shell. Every non-system input is fetched and pinned below.
export PATH="/usr/bin:/bin:/usr/sbin:/sbin"
readonly PATH

readonly TESSERACT_REPOSITORY="https://github.com/tesseract-ocr/tesseract.git"
readonly TESSERACT_COMMIT="6e1d56a847e697de07b38619356550e5cf4e8633"
readonly TESSERACT_LICENSE_SHA256="cfc7749b96f63bd31c3c42b5c471bf756814053e847c10f3eb003417bc523d30"
readonly LEPTONICA_REPOSITORY="https://github.com/DanBloomberg/leptonica.git"
readonly LEPTONICA_COMMIT="63aef18d98432b8582a1565e241f7bd2ee9cc8d9"
readonly LEPTONICA_LICENSE_SHA256="87829abb5bbb00b55a107365da89e9a33f86c4250169e5a1e5588505be7d5806"
readonly ZLIB_REPOSITORY="https://github.com/madler/zlib.git"
readonly ZLIB_COMMIT="51b7f2abdade71cd9bb0e7a373ef2610ec6f9daf"
readonly ZLIB_LICENSE_SHA256="845efc77857d485d91fb3e0b884aaa929368c717ae8186b66fe1ed2495753243"
readonly LIBPNG_REPOSITORY="https://github.com/pnggroup/libpng.git"
readonly LIBPNG_COMMIT="2b978915d82377df13fcbb1fb56660195ded868a"
readonly LIBPNG_LICENSE_SHA256="16d9daaafbf63a31a5bdc91d4600972548fef5aaa1244202393288dbd079c49a"
readonly TESSDATA_REPOSITORY="https://github.com/tesseract-ocr/tessdata_fast.git"
readonly TESSDATA_COMMIT="87416418657359cb625c412a48b6e1d6d41c29bd"
readonly TESSDATA_LICENSE_SHA256="cfc7749b96f63bd31c3c42b5c471bf756814053e847c10f3eb003417bc523d30"
readonly ENGLISH_MODEL_SHA256="7d4322bd2a7749724879683fc3912cb542f19906c83bcc1a52132556427170b2"
readonly RUSSIAN_MODEL_SHA256="e16e5e036cce1d9ec2b00063cf8b54472625b9e14d893a169e2b0dedeb4df225"
readonly CMAKE_VERSION="4.4.2"
readonly CMAKE_ARCHIVE_SHA256="800fc86838e913fff969b499886c80baeb4ccfd00f0e39906b34aa334f39ab6c"
readonly XCODE_VERSION="26.6"
readonly MACOS_SDK_VERSION="26.5"
readonly MACOS_SDK_BUILD="25F70"
readonly CLANG_SHA256="7def90dd8829726686213a747fc5bff1583df933dae5edc55d755479e0bfe00a"
readonly LINKER_SHA256="5897b275efd93b201b6df5832dd541262b3f20f290859ba78f2200a6a66ef38b"
readonly DEPLOYMENT_TARGET="13.0"
readonly RUNNER_NAME="tesseract-runner"

usage() {
  echo "usage: $0 --output-dir <new-absolute-directory> [--verify-reproducibility]" >&2
}

output_directory=""
verify_reproducibility=false
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
    --verify-reproducibility)
      if [[ "$verify_reproducibility" == true ]]; then
        usage
        exit 2
      fi
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
  echo "the pinned OCR release target requires macOS arm64" >&2
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
  echo "the active Xcode toolchain does not match the pinned OCR release toolchain" >&2
  exit 1
fi

scratch_directory="$(mktemp -d "${TMPDIR:-/tmp}/makosh-ocr-build.XXXXXXXX")"
trap 'rm -rf -- "$scratch_directory"' EXIT
source_directory="${scratch_directory}/sources"
mkdir "$source_directory"

clone_exact() {
  local repository="$1"
  local commit="$2"
  local destination="$3"
  git init -q "$destination"
  git -C "$destination" remote add origin "$repository"
  git -C "$destination" fetch -q --depth 1 origin "$commit"
  git -C "$destination" -c advice.detachedHead=false checkout -q --detach "$commit"
  if [[ "$(git -C "$destination" rev-parse HEAD)" != "$commit" ]]; then
    echo "an OCR source checkout does not match its pinned commit" >&2
    exit 1
  fi
}

clone_exact "$TESSERACT_REPOSITORY" "$TESSERACT_COMMIT" "${source_directory}/tesseract"
clone_exact "$LEPTONICA_REPOSITORY" "$LEPTONICA_COMMIT" "${source_directory}/leptonica"
clone_exact "$ZLIB_REPOSITORY" "$ZLIB_COMMIT" "${source_directory}/zlib"
clone_exact "$LIBPNG_REPOSITORY" "$LIBPNG_COMMIT" "${source_directory}/libpng"
clone_exact "$TESSDATA_REPOSITORY" "$TESSDATA_COMMIT" "${source_directory}/tessdata"

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

verify_sha256 "${source_directory}/tesseract/LICENSE" "$TESSERACT_LICENSE_SHA256" "Tesseract license"
verify_sha256 "${source_directory}/leptonica/leptonica-license.txt" "$LEPTONICA_LICENSE_SHA256" "Leptonica license"
verify_sha256 "${source_directory}/zlib/LICENSE" "$ZLIB_LICENSE_SHA256" "zlib license"
verify_sha256 "${source_directory}/libpng/LICENSE" "$LIBPNG_LICENSE_SHA256" "libpng license"
verify_sha256 "${source_directory}/tessdata/LICENSE" "$TESSDATA_LICENSE_SHA256" "tessdata license"
verify_sha256 "${source_directory}/tessdata/eng.traineddata" "$ENGLISH_MODEL_SHA256" "English model"
verify_sha256 "${source_directory}/tessdata/rus.traineddata" "$RUSSIAN_MODEL_SHA256" "Russian model"

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

source_date_epoch="$(git -C "${source_directory}/tesseract" show -s --format=%ct HEAD)"
readonly source_date_epoch
common_c_flags="-O2 -DNDEBUG -isysroot ${sdk_path}"
common_linker_flags=""
readonly common_c_flags common_linker_flags

configure() {
  local source="$1"
  local build="$2"
  local isolated_build_root
  local mapped_c_flags
  shift 2
  isolated_build_root="$(dirname "$build")"
  mapped_c_flags="${common_c_flags} -ffile-prefix-map=${source_directory}=/usr/src/makosh-ocr/sources -fdebug-prefix-map=${source_directory}=/usr/src/makosh-ocr/sources -ffile-prefix-map=${isolated_build_root}=/usr/src/makosh-ocr/build -fdebug-prefix-map=${isolated_build_root}=/usr/src/makosh-ocr/build"
  "$cmake_path" -S "$source" -B "$build" \
    -G "Unix Makefiles" \
    -DCMAKE_MAKE_PROGRAM=/usr/bin/make \
    -DCMAKE_BUILD_TYPE=Release \
    -DCMAKE_OSX_ARCHITECTURES=arm64 \
    -DCMAKE_OSX_DEPLOYMENT_TARGET="$DEPLOYMENT_TARGET" \
    -DCMAKE_OSX_SYSROOT="$sdk_path" \
    -DCMAKE_C_COMPILER="$clang_path" \
    -DCMAKE_CXX_COMPILER="$clangxx_path" \
    -DCMAKE_C_FLAGS_RELEASE="$mapped_c_flags" \
    -DCMAKE_CXX_FLAGS_RELEASE="${mapped_c_flags} -stdlib=libc++" \
    -DCMAKE_EXE_LINKER_FLAGS="$common_linker_flags" \
    -DCMAKE_SHARED_LINKER_FLAGS="$common_linker_flags" \
    -DCMAKE_FIND_USE_PACKAGE_REGISTRY=OFF \
    -DCMAKE_FIND_USE_SYSTEM_PACKAGE_REGISTRY=OFF \
    -DCMAKE_FIND_USE_SYSTEM_ENVIRONMENT_PATH=OFF \
    -DCMAKE_IGNORE_PATH="/opt/homebrew/bin;/opt/homebrew/sbin;/usr/local/bin;/usr/local/sbin" \
    -DCMAKE_IGNORE_PREFIX_PATH="/opt/homebrew;/usr/local" \
    "$@"
}

build_once() {
  local build_root="$1"
  local release_root="$2"
  local prefix="${build_root}/prefix"
  mkdir -p "$build_root"
  export SOURCE_DATE_EPOCH="$source_date_epoch"
  export ZERO_AR_DATE=1
  export LC_ALL=C
  export LANG=C
  export TZ=UTC
  export SDKROOT="$sdk_path"

  configure "${source_directory}/zlib" "${build_root}/zlib" \
    -DCMAKE_INSTALL_PREFIX="$prefix" \
    -DBUILD_SHARED_LIBS=OFF \
    -DZLIB_BUILD_EXAMPLES=OFF
  "$cmake_path" --build "${build_root}/zlib" --target install --parallel

  configure "${source_directory}/libpng" "${build_root}/libpng" \
    -DCMAKE_INSTALL_PREFIX="$prefix" \
    -DPNG_SHARED=OFF \
    -DPNG_STATIC=ON \
    -DPNG_TESTS=OFF \
    -DPNG_TOOLS=OFF \
    -DPNG_FRAMEWORK=OFF \
    -DAWK=/usr/bin/awk \
    -DZLIB_LIBRARY="${prefix}/lib/libz.a" \
    -DZLIB_INCLUDE_DIR="$prefix/include"
  "$cmake_path" --build "${build_root}/libpng" --target install --parallel

  configure "${source_directory}/leptonica" "${build_root}/leptonica" \
    -DCMAKE_INSTALL_PREFIX="$prefix" \
    -DCMAKE_PREFIX_PATH="$prefix" \
    -DBUILD_SHARED_LIBS=OFF \
    -DBUILD_PROG=OFF \
    -DSW_BUILD=OFF \
    -DENABLE_ZLIB=ON \
    -DENABLE_PNG=ON \
    -DENABLE_GIF=OFF \
    -DENABLE_JPEG=OFF \
    -DENABLE_TIFF=OFF \
    -DENABLE_WEBP=OFF \
    -DENABLE_OPENJPEG=OFF \
    -DZLIB_LIBRARY="${prefix}/lib/libz.a" \
    -DZLIB_INCLUDE_DIR="$prefix/include" \
    -DPNG_LIBRARY="${prefix}/lib/libpng16.a" \
    -DPNG_PNG_INCLUDE_DIR="$prefix/include"
  "$cmake_path" --build "${build_root}/leptonica" --target install --parallel

  configure "${source_directory}/tesseract" "${build_root}/tesseract" \
    -DCMAKE_INSTALL_PREFIX="$prefix" \
    -DCMAKE_PREFIX_PATH="$prefix" \
    -DBUILD_SHARED_LIBS=OFF \
    -DSW_BUILD=OFF \
    -DOPENMP_BUILD=OFF \
    -DGRAPHICS_DISABLED=ON \
    -DBUILD_TRAINING_TOOLS=OFF \
    -DBUILD_TESTS=OFF \
    -DDISABLE_ARCHIVE=ON \
    -DDISABLE_CURL=ON \
    -DDISABLE_TIFF=ON \
    -DLeptonica_DIR="${prefix}/lib/cmake/leptonica"
  "$cmake_path" --build "${build_root}/tesseract" --target tesseract --parallel

  mkdir "$release_root"
  install -m 0555 "${build_root}/tesseract/bin/tesseract" "${release_root}/${RUNNER_NAME}"
  install -m 0444 "${source_directory}/tessdata/eng.traineddata" "${release_root}/eng.traineddata"
  install -m 0444 "${source_directory}/tessdata/rus.traineddata" "${release_root}/rus.traineddata"
  install -m 0444 "${source_directory}/tesseract/LICENSE" "${release_root}/LICENSE.tesseract-Apache-2.0"
  install -m 0444 "${source_directory}/leptonica/leptonica-license.txt" "${release_root}/LICENSE.leptonica-BSD-2-Clause"
  install -m 0444 "${source_directory}/zlib/LICENSE" "${release_root}/LICENSE.zlib-Zlib"
  install -m 0444 "${source_directory}/libpng/LICENSE" "${release_root}/LICENSE.libpng-Libpng"
  install -m 0444 "${source_directory}/tessdata/LICENSE" "${release_root}/LICENSE.tessdata-Apache-2.0"

  local non_system_loads
  non_system_loads="$(otool -L "${release_root}/${RUNNER_NAME}" | tail -n +2 | awk '{print $1}' \
    | grep -Ev '^(/usr/lib/|/System/Library/)' || true)"
  if [[ -n "$non_system_loads" ]]; then
    echo "the OCR runner has a non-system dynamic dependency" >&2
    exit 1
  fi
  if otool -l "${release_root}/${RUNNER_NAME}" | grep -q 'LC_RPATH'; then
    echo "the OCR runner must not contain an rpath" >&2
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
  for artifact in "$RUNNER_NAME" eng.traineddata rus.traineddata dynamic-dependencies.txt; do
    if ! cmp -s "${first_release}/${artifact}" "${second_release}/${artifact}"; then
      echo "the isolated OCR builds are not reproducible" >&2
      exit 1
    fi
  done
  reproducibility_verified=true
fi

cat >"${first_release}/provenance.json" <<EOF
{
  "artifact": "${RUNNER_NAME}",
  "artifact_sha256": "${runner_sha}",
  "clang_sha256": "${CLANG_SHA256}",
  "cmake_archive_sha256": "${CMAKE_ARCHIVE_SHA256}",
  "cmake_version": "${CMAKE_VERSION}",
  "deployment_target": "${DEPLOYMENT_TARGET}",
  "english_model_sha256": "${ENGLISH_MODEL_SHA256}",
  "leptonica_commit": "${LEPTONICA_COMMIT}",
  "libpng_commit": "${LIBPNG_COMMIT}",
  "linker_sha256": "${LINKER_SHA256}",
  "macos_sdk_build": "${MACOS_SDK_BUILD}",
  "macos_sdk_version": "${MACOS_SDK_VERSION}",
  "platform": "darwin-arm64",
  "release_eligible": ${reproducibility_verified},
  "reproducibility_verified": ${reproducibility_verified},
  "russian_model_sha256": "${RUSSIAN_MODEL_SHA256}",
  "source_date_epoch": ${source_date_epoch},
  "tessdata_commit": "${TESSDATA_COMMIT}",
  "tesseract_commit": "${TESSERACT_COMMIT}",
  "xcode_version": "${XCODE_VERSION}",
  "zlib_commit": "${ZLIB_COMMIT}"
}
EOF
chmod 0444 "${first_release}/provenance.json"

mv "$first_release" "$output_directory"
trap - EXIT
rm -rf -- "$scratch_directory"
echo "built ${output_directory}/${RUNNER_NAME}"
