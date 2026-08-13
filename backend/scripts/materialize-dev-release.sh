#!/usr/bin/env bash

set -euo pipefail
umask 077

backend_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
project_root="$(cd "$backend_root/.." && pwd)"
frontend_root="$project_root/frontend"
local_root="$project_root/.local"
cargo_target_dir="${MAKOSH_DEV_CARGO_TARGET_DIR:-$backend_root/target}"
release_root="${MAKOSH_DEV_RELEASE_ROOT:-$local_root/dev-release}"
signing_key="${MAKOSH_DEV_RELEASE_SIGNING_KEY:-$local_root/dev-release-signing-key.pem}"
tgcalls_root="${MAKOSH_DEV_TGCALLS_ROOT:-$local_root/dev-native/tgcalls}"
attachment_text_extraction_ocr_root="${MAKOSH_DEV_ATTACHMENT_TEXT_EXTRACTION_OCR_ROOT:-$local_root/dev-native/attachment-text-extraction-ocr}"
whisper_stt_root="${MAKOSH_DEV_WHISPER_STT_ROOT:-$local_root/dev-native/whisper-stt}"
distribution_id="makosh-local-development"
distribution_generation=""
generation_metadata_name="development-distribution-generation"
release_version="1"
build_id="local-development"
target_triple="aarch64-apple-darwin"
staging_root=""

fail() {
	printf 'Макошь development release failed: %s\n' "$1" >&2
	exit 1
}

require_command() {
	command -v "$1" >/dev/null 2>&1 || fail "required command '$1' is unavailable"
}

require_absolute_path() {
	case "$2" in
		/*) ;;
		*) fail "$1 must be an absolute path" ;;
	esac
}

require_regular_file() {
	test -f "$1" && test ! -L "$1" || fail "$2 must be a regular non-symlink file"
}

next_distribution_generation() {
	if ! test -e "$release_root"; then
		printf '%s\n' 1
		return
	fi
	test -d "$release_root" && test ! -L "$release_root" \
		|| fail "existing development release root is invalid"
	metadata_path="$release_root/$generation_metadata_name"
	if ! test -e "$metadata_path"; then
		printf '%s\n' 2
		return
	fi
	require_regular_file "$metadata_path" "development release generation metadata"
	test "$(stat -f '%Lp' "$metadata_path")" = "600" \
		|| fail "development release generation metadata permissions must be 0600"
	installed_generation="$(sed -n '1p' "$metadata_path")"
	test "$(wc -l <"$metadata_path" | tr -d ' ')" = "1" \
		|| fail "development release generation metadata is invalid"
	case "$installed_generation" in
		''|*[!0-9]*) fail "development release generation metadata is invalid" ;;
	esac
	test "$installed_generation" -gt 0 \
		|| fail "development release generation metadata is invalid"
	test "$installed_generation" -lt 9007199254740991 \
		|| fail "development release generation cannot advance"
	printf '%s\n' "$((installed_generation + 1))"
}

remove_staging_root() {
	test -n "$staging_root" || return 0
	case "$staging_root" in
		"$local_root"/dev-release-staging.*)
			rm -rf -- "$staging_root"
			;;
		*)
			fail "refusing to remove an unexpected staging path"
			;;
	esac
	staging_root=""
}

cleanup() {
	status=$?
	trap - EXIT INT TERM HUP
	remove_staging_root
	exit "$status"
}

sha256_file() {
	shasum -a 256 "$1" | awk '{print $1}'
}

trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM
trap 'exit 129' HUP

for command_name in awk brew cargo git mktemp node pnpm rustc shasum uname; do
	require_command "$command_name"
done
require_absolute_path "MAKOSH_DEV_CARGO_TARGET_DIR" "$cargo_target_dir"
require_absolute_path "MAKOSH_DEV_RELEASE_ROOT" "$release_root"
require_absolute_path "MAKOSH_DEV_RELEASE_SIGNING_KEY" "$signing_key"
require_absolute_path "MAKOSH_DEV_TGCALLS_ROOT" "$tgcalls_root"
require_absolute_path \
	"MAKOSH_DEV_ATTACHMENT_TEXT_EXTRACTION_OCR_ROOT" \
	"$attachment_text_extraction_ocr_root"
require_absolute_path "MAKOSH_DEV_WHISPER_STT_ROOT" "$whisper_stt_root"
test "$(uname -m)" = "arm64" || fail "the current development release supports macOS arm64 only"

mkdir -p "$local_root"
chmod 700 "$local_root"
distribution_generation="$(next_distribution_generation)"

tdlib_prefix="$(brew --prefix tdlib 2>/dev/null)" \
	|| fail "Homebrew TDLib is required for the Telegram runtime"
tdlib_library_dir="$(cd "$tdlib_prefix/lib" && pwd -P)"
tdjson_candidates=("$tdlib_library_dir"/libtdjson.*.dylib)
test "${#tdjson_candidates[@]}" -eq 1 \
	|| fail "exactly one canonical versioned TDLib dylib is required"
tdjson_path="${tdjson_candidates[0]}"
require_regular_file "$tdjson_path" "TDLib dylib"

tgcalls_path="$tgcalls_root/libmakosh_tgcalls_bridge.dylib"
if ! test -f "$tgcalls_path"; then
	printf '%s\n' 'Building the pinned Telegram call bridge for local development...' >&2
	"$backend_root/scripts/build-telegram-tgcalls-bridge-macos.sh" \
		--output-dir "$tgcalls_root" \
		--development-audio-conformance
fi
require_regular_file "$tgcalls_path" "Telegram call bridge"

attachment_text_extraction_ocr_runner="$attachment_text_extraction_ocr_root/tesseract-runner"
attachment_text_extraction_ocr_english="$attachment_text_extraction_ocr_root/eng.traineddata"
attachment_text_extraction_ocr_russian="$attachment_text_extraction_ocr_root/rus.traineddata"
if ! test -e "$attachment_text_extraction_ocr_root"; then
	printf '%s\n' 'Building the pinned Attachment Text Extraction OCR runtime for local development...' >&2
	"$backend_root/scripts/build-attachment-text-extraction-ocr-macos.sh" \
		--output-dir "$attachment_text_extraction_ocr_root"
fi
test -d "$attachment_text_extraction_ocr_root" \
	&& test ! -L "$attachment_text_extraction_ocr_root" \
	|| fail "Attachment Text Extraction OCR runtime root is invalid"
require_regular_file \
	"$attachment_text_extraction_ocr_runner" \
	"Attachment Text Extraction OCR runner"
require_regular_file \
	"$attachment_text_extraction_ocr_english" \
	"Attachment Text Extraction English OCR model"
require_regular_file \
	"$attachment_text_extraction_ocr_russian" \
	"Attachment Text Extraction Russian OCR model"

whisper_stt_runner="$whisper_stt_root/whisper-cli"
whisper_stt_model="$whisper_stt_root/ggml-base.bin"
if ! test -e "$whisper_stt_root"; then
	printf '%s\n' 'Building the pinned Whisper STT runtime for local development...' >&2
	"$backend_root/scripts/build-whisper-stt-macos.sh" \
		--output-dir "$whisper_stt_root"
fi
test -d "$whisper_stt_root" && test ! -L "$whisper_stt_root" \
	|| fail "Whisper STT runtime root is invalid"
require_regular_file "$whisper_stt_runner" "Whisper STT runner"
require_regular_file "$whisper_stt_model" "Whisper STT model"

printf '%s\n' 'Building signed-development runtime and assembly units...' >&2
CARGO_TARGET_DIR="$cargo_target_dir" cargo +1.97.0 build --locked \
	--package makosh-kernel \
	--package makosh-blob-service \
	--package makosh-events-authority-runtime \
	--package makosh-scheduler-runtime \
	--package makosh-storage-runtime \
	--package makosh-telemetry-collector \
	--package makosh-vault-runtime \
	--package makosh-communications-runtime \
	--package makosh-communications-assembly \
	--package makosh-communications-export-runtime \
	--package makosh-communications-export-assembly \
	--package makosh-communication-delivery-intent-runtime \
	--package makosh-communication-delivery-intent-assembly \
	--package makosh-communication-bulk-action-runtime \
	--package makosh-communication-bulk-action-assembly \
	--package makosh-communication-cross-channel-forward-runtime \
	--package makosh-communication-cross-channel-forward-assembly \
	--package makosh-communication-reply-suggestion-runtime \
	--package makosh-communication-reply-suggestion-assembly \
	--package makosh-communication-summary-runtime \
	--package makosh-communication-summary-assembly \
	--package makosh-communication-translation-runtime \
	--package makosh-communication-translation-assembly \
	--package makosh-communication-explanation-runtime \
	--package makosh-communication-explanation-assembly \
	--package makosh-communication-recipient-suggestion-runtime \
	--package makosh-communication-recipient-suggestion-assembly \
	--package makosh-communication-task-candidate-runtime \
	--package makosh-communication-task-candidate-assembly \
	--package makosh-communication-note-candidate-runtime \
	--package makosh-communication-note-candidate-assembly \
	--package makosh-review-task-candidate-runtime \
	--package makosh-review-task-candidate-assembly \
	--package makosh-tasks-runtime \
	--package makosh-tasks-assembly \
	--package makosh-review-obligation-candidate-runtime \
	--package makosh-review-obligation-candidate-assembly \
	--package makosh-obligations-runtime \
	--package makosh-obligations-assembly \
	--package makosh-reviewed-obligation-candidate-promotion-runtime \
	--package makosh-reviewed-obligation-candidate-promotion-assembly \
	--package makosh-persons-runtime \
	--package makosh-persons-assembly \
	--package makosh-identity-resolution-runtime \
	--package makosh-identity-resolution-assembly \
	--package makosh-mail-persons-sync-runtime \
	--package makosh-mail-persons-sync-assembly \
	--package makosh-review-person-match-candidate-runtime \
	--package makosh-review-person-match-candidate-assembly \
	--package makosh-reviewed-person-match-candidate-promotion-runtime \
	--package makosh-reviewed-person-match-candidate-promotion-assembly \
	--package makosh-knowledge-runtime \
	--package makosh-knowledge-assembly \
	--package makosh-review-note-candidate-runtime \
	--package makosh-review-note-candidate-assembly \
	--package makosh-reviewed-note-candidate-promotion-runtime \
	--package makosh-reviewed-note-candidate-promotion-assembly \
	--package makosh-reviewed-task-candidate-promotion-runtime \
	--package makosh-reviewed-task-candidate-promotion-assembly \
	--package makosh-communication-delayed-delivery-runtime \
	--package makosh-communication-delayed-delivery-assembly \
	--package makosh-attachment-security-runtime \
	--package makosh-attachment-security-assembly \
	--package makosh-attachment-text-extraction-runtime \
	--package makosh-attachment-text-extraction-assembly \
	--package makosh-attachment-preview-runtime \
	--package makosh-attachment-preview-assembly \
	--package makosh-attachment-preview-evidence-replay-runtime \
	--package makosh-attachment-preview-evidence-replay-assembly \
	--package makosh-attachment-translation-runtime \
	--package makosh-attachment-translation-assembly \
	--package makosh-ai-inference-runtime \
	--package makosh-ai-inference-assembly \
	--package makosh-ollama-ai-runtime \
	--package makosh-ollama-ai-assembly \
	--package makosh-speech-to-text-runtime \
	--package makosh-speech-to-text-assembly \
	--package makosh-whisper-stt-runtime \
	--package makosh-whisper-stt-assembly \
	--package makosh-calendar-runtime \
	--package makosh-calendar-assembly \
	--package makosh-organizations-runtime \
	--package makosh-organizations-assembly \
	--package makosh-documents-runtime \
	--package makosh-documents-assembly \
	--package makosh-relationships-runtime \
	--package makosh-relationships-assembly \
	--package makosh-projects-runtime \
	--package makosh-projects-assembly \
	--package makosh-decisions-runtime \
	--package makosh-decisions-assembly \
	--package makosh-search-runtime \
	--package makosh-search-assembly \
	--package makosh-timeline-runtime \
	--package makosh-timeline-assembly \
	--package makosh-graph-runtime \
	--package makosh-graph-assembly \
	--package makosh-memory-runtime \
	--package makosh-memory-assembly \
	--package makosh-consistency-runtime \
	--package makosh-consistency-assembly \
	--package makosh-risk-runtime \
	--package makosh-risk-assembly \
	--package makosh-zoom-runtime \
	--package makosh-zoom-assembly \
	--package makosh-telemost-runtime \
	--package makosh-telemost-assembly \
	--package makosh-omniroute-runtime \
	--package makosh-omniroute-assembly \
	--package makosh-desktop-call-recording-runtime \
	--package makosh-desktop-call-recording-assembly \
	--package makosh-mail-runtime \
	--package makosh-mail-assembly \
	--package makosh-telegram-runtime \
	--package makosh-telegram-assembly \
	--package makosh-whatsapp-runtime \
	--package makosh-whatsapp-assembly \
	--package makosh-zulip-runtime \
	--package makosh-zulip-assembly \
	--package makosh-development-assembly

printf '%s\n' 'Building the Vue browser client for the signed development bundle...' >&2
(
	cd "$frontend_root"
	pnpm build
)

staging_root="$(mktemp -d "$local_root/dev-release-staging.XXXXXX")"
chmod 700 "$staging_root"
scratch_root="$staging_root/scratch"
assembly_root="$staging_root/assemblies"
new_release_root="$staging_root/release"
app_root="$new_release_root/МакошьDev.app"
resource_root="$app_root/Contents/Resources/makosh-kernel-release"
mkdir -p \
	"$scratch_root/descriptors" \
	"$assembly_root" \
	"$app_root/Contents/MacOS" \
	"$resource_root"

source_commit="$(git -C "$project_root" rev-parse HEAD)"
lockfile_sha256="$(sha256_file "$backend_root/Cargo.lock")"
sbom_path="$scratch_root/cargo-metadata.json"
toolchain_path="$scratch_root/toolchain.txt"
(
	cd "$backend_root"
	cargo +1.97.0 metadata --locked --format-version 1
) >"$sbom_path"
{
	rustc +1.97.0 -vV
	cargo +1.97.0 -vV
} >"$toolchain_path"
sbom_sha256="$(sha256_file "$sbom_path")"
toolchain_sha256="$(sha256_file "$toolchain_path")"

communications_assembly="$assembly_root/communications"
communications_export_assembly="$assembly_root/communications-export"
communication_delivery_intent_assembly="$assembly_root/communication-delivery-intent"
communication_bulk_action_assembly="$assembly_root/communication-bulk-action"
communication_cross_channel_forward_assembly="$assembly_root/communication-cross-channel-forward"
communication_reply_suggestion_assembly="$assembly_root/communication-reply-suggestion"
communication_summary_assembly="$assembly_root/communication-summary"
communication_translation_assembly="$assembly_root/communication-translation"
communication_explanation_assembly="$assembly_root/communication-explanation"
communication_recipient_suggestion_assembly="$assembly_root/communication-recipient-suggestion"
communication_task_candidate_assembly="$assembly_root/communication-task-candidate"
communication_note_candidate_assembly="$assembly_root/communication-note-candidate"
review_task_candidate_assembly="$assembly_root/review-task-candidate"
tasks_assembly="$assembly_root/tasks"
review_obligation_candidate_assembly="$assembly_root/review-obligation-candidate"
obligations_assembly="$assembly_root/obligations"
reviewed_obligation_candidate_promotion_assembly="$assembly_root/reviewed-obligation-candidate-promotion"
persons_assembly="$assembly_root/persons"
identity_resolution_assembly="$assembly_root/identity-resolution"
mail_persons_sync_assembly="$assembly_root/mail-persons-sync"
review_person_match_candidate_assembly="$assembly_root/review-person-match-candidate"
reviewed_person_match_candidate_promotion_assembly="$assembly_root/reviewed-person-match-candidate-promotion"
knowledge_assembly="$assembly_root/knowledge"
review_note_candidate_assembly="$assembly_root/review-note-candidate"
reviewed_note_candidate_promotion_assembly="$assembly_root/reviewed-note-candidate-promotion"
reviewed_task_candidate_promotion_assembly="$assembly_root/reviewed-task-candidate-promotion"
communication_delayed_delivery_assembly="$assembly_root/communication-delayed-delivery"
attachment_security_assembly="$assembly_root/attachment-security"
attachment_text_extraction_assembly="$assembly_root/attachment-text-extraction"
attachment_preview_assembly="$assembly_root/attachment-preview"
attachment_preview_evidence_replay_assembly="$assembly_root/attachment-preview-evidence-replay"
attachment_translation_assembly="$assembly_root/attachment-translation"
ai_inference_assembly="$assembly_root/ai-inference"
ollama_ai_assembly="$assembly_root/ollama-ai"
speech_to_text_assembly="$assembly_root/speech-to-text"
whisper_stt_assembly="$assembly_root/whisper-stt"
calendar_assembly="$assembly_root/calendar"
organizations_assembly="$assembly_root/organizations"
documents_assembly="$assembly_root/documents"
relationships_assembly="$assembly_root/relationships"
projects_assembly="$assembly_root/projects"
decisions_assembly="$assembly_root/decisions"
search_assembly="$assembly_root/search"
timeline_assembly="$assembly_root/timeline"
graph_assembly="$assembly_root/graph"
memory_assembly="$assembly_root/memory"
consistency_assembly="$assembly_root/consistency"
risk_assembly="$assembly_root/risk"
zoom_assembly="$assembly_root/zoom"
telemost_assembly="$assembly_root/telemost"
omniroute_assembly="$assembly_root/omniroute"
desktop_call_recording_assembly="$assembly_root/desktop-call-recording"
mail_assembly="$assembly_root/mail"
telegram_assembly="$assembly_root/telegram"
whatsapp_assembly="$assembly_root/whatsapp"
zulip_assembly="$assembly_root/zulip"

"$cargo_target_dir/debug/makosh-communications-assembly" \
	--build-id "$build_id" \
	--output-dir "$communications_assembly" \
	--runtime "$cargo_target_dir/debug/makosh-communications-runtime"
"$cargo_target_dir/debug/makosh-communications-export-assembly" \
	--build-id "$build_id" \
	--output-dir "$communications_export_assembly" \
	--runtime "$cargo_target_dir/debug/makosh-communications-export-runtime"
"$cargo_target_dir/debug/makosh-communication-delivery-intent-assembly" \
	--build-id "$build_id" \
	--output-dir "$communication_delivery_intent_assembly" \
	--runtime "$cargo_target_dir/debug/makosh-communication-delivery-intent-runtime"
"$cargo_target_dir/debug/makosh-communication-bulk-action-assembly" \
	--build-id "$build_id" \
	--output-dir "$communication_bulk_action_assembly" \
	--runtime "$cargo_target_dir/debug/makosh-communication-bulk-action-runtime"
"$cargo_target_dir/debug/makosh-communication-cross-channel-forward-assembly" \
	--build-id "$build_id" \
	--output-dir "$communication_cross_channel_forward_assembly" \
	--runtime "$cargo_target_dir/debug/makosh-communication-cross-channel-forward-runtime"
"$cargo_target_dir/debug/makosh-communication-reply-suggestion-assembly" \
	--build-id "$build_id" \
	--output-dir "$communication_reply_suggestion_assembly" \
	--runtime "$cargo_target_dir/debug/makosh-communication-reply-suggestion-runtime"
"$cargo_target_dir/debug/makosh-communication-summary-assembly" \
	--build-id "$build_id" \
	--output-dir "$communication_summary_assembly" \
	--runtime "$cargo_target_dir/debug/makosh-communication-summary-runtime"
"$cargo_target_dir/debug/makosh-communication-translation-assembly" \
	--build-id "$build_id" \
	--output-dir "$communication_translation_assembly" \
	--runtime "$cargo_target_dir/debug/makosh-communication-translation-runtime"
"$cargo_target_dir/debug/makosh-communication-explanation-assembly" \
	--build-id "$build_id" \
	--output-dir "$communication_explanation_assembly" \
	--runtime "$cargo_target_dir/debug/makosh-communication-explanation-runtime"
"$cargo_target_dir/debug/makosh-communication-recipient-suggestion-assembly" \
	--build-id "$build_id" \
	--output-dir "$communication_recipient_suggestion_assembly" \
	--runtime "$cargo_target_dir/debug/makosh-communication-recipient-suggestion-runtime"
"$cargo_target_dir/debug/makosh-communication-task-candidate-assembly" \
	--build-id "$build_id" \
	--output-dir "$communication_task_candidate_assembly" \
	--runtime "$cargo_target_dir/debug/makosh-communication-task-candidate-runtime"
"$cargo_target_dir/debug/makosh-communication-note-candidate-assembly" \
	--build-id "$build_id" \
	--output-dir "$communication_note_candidate_assembly" \
	--runtime "$cargo_target_dir/debug/makosh-communication-note-candidate-runtime"
"$cargo_target_dir/debug/makosh-review-task-candidate-assembly" \
	--build-id "$build_id" \
	--output-dir "$review_task_candidate_assembly" \
	--runtime "$cargo_target_dir/debug/makosh-review-task-candidate-runtime"
"$cargo_target_dir/debug/makosh-tasks-assembly" \
	--build-id "$build_id" \
	--output-dir "$tasks_assembly" \
	--runtime "$cargo_target_dir/debug/makosh-tasks-runtime"
"$cargo_target_dir/debug/makosh-review-obligation-candidate-assembly" \
	--build-id "$build_id" \
	--output-dir "$review_obligation_candidate_assembly" \
	--runtime "$cargo_target_dir/debug/makosh-review-obligation-candidate-runtime"
"$cargo_target_dir/debug/makosh-obligations-assembly" \
	--build-id "$build_id" \
	--output-dir "$obligations_assembly" \
	--runtime "$cargo_target_dir/debug/makosh-obligations-runtime"
"$cargo_target_dir/debug/makosh-reviewed-obligation-candidate-promotion-assembly" \
	--build-id "$build_id" \
	--output-dir "$reviewed_obligation_candidate_promotion_assembly" \
	--runtime "$cargo_target_dir/debug/makosh-reviewed-obligation-candidate-promotion-runtime"
"$cargo_target_dir/debug/makosh-persons-assembly" \
	--build-id "$build_id" \
	--runtime "$cargo_target_dir/debug/makosh-persons-runtime" \
	--output "$persons_assembly"
"$cargo_target_dir/debug/makosh-identity-resolution-assembly" \
	--build-id "$build_id" \
	--output-dir "$identity_resolution_assembly" \
	--runtime "$cargo_target_dir/debug/makosh-identity-resolution-runtime"
"$cargo_target_dir/debug/makosh-mail-persons-sync-assembly" \
	--build-id "$build_id" \
	--runtime "$cargo_target_dir/debug/makosh-mail-persons-sync-runtime" \
	--output "$mail_persons_sync_assembly"
"$cargo_target_dir/debug/makosh-review-person-match-candidate-assembly" assemble \
	--output-dir "$review_person_match_candidate_assembly" \
	--build-id "$build_id" \
	--runtime-source "$cargo_target_dir/debug/makosh-review-person-match-candidate-runtime"
"$cargo_target_dir/debug/makosh-reviewed-person-match-candidate-promotion-assembly" assemble \
	--output-dir "$reviewed_person_match_candidate_promotion_assembly" \
	--build-id "$build_id" \
	--runtime-source "$cargo_target_dir/debug/makosh-reviewed-person-match-candidate-promotion-runtime"
"$cargo_target_dir/debug/makosh-knowledge-assembly" \
	--build-id "$build_id" \
	--output-dir "$knowledge_assembly" \
	--runtime "$cargo_target_dir/debug/makosh-knowledge-runtime"
"$cargo_target_dir/debug/makosh-review-note-candidate-assembly" \
	--build-id "$build_id" \
	--output-dir "$review_note_candidate_assembly" \
	--runtime "$cargo_target_dir/debug/makosh-review-note-candidate-runtime"
"$cargo_target_dir/debug/makosh-reviewed-note-candidate-promotion-assembly" \
	--build-id "$build_id" \
	--output-dir "$reviewed_note_candidate_promotion_assembly" \
	--runtime "$cargo_target_dir/debug/makosh-reviewed-note-candidate-promotion-runtime"
"$cargo_target_dir/debug/makosh-reviewed-task-candidate-promotion-assembly" \
	--build-id "$build_id" \
	--output-dir "$reviewed_task_candidate_promotion_assembly" \
	--runtime "$cargo_target_dir/debug/makosh-reviewed-task-candidate-promotion-runtime"
"$cargo_target_dir/debug/makosh-communication-delayed-delivery-assembly" \
	--build-id "$build_id" \
	--output-dir "$communication_delayed_delivery_assembly" \
	--runtime "$cargo_target_dir/debug/makosh-communication-delayed-delivery-runtime"
"$cargo_target_dir/debug/makosh-attachment-security-assembly" \
	--build-id "$build_id" \
	--output-dir "$attachment_security_assembly" \
	--runtime "$cargo_target_dir/debug/makosh-attachment-security-runtime"
"$cargo_target_dir/debug/makosh-attachment-text-extraction-assembly" \
	--build-id "$build_id" \
	--output-dir "$attachment_text_extraction_assembly" \
	--runtime "$cargo_target_dir/debug/makosh-attachment-text-extraction-runtime" \
	--ocr-runner "$attachment_text_extraction_ocr_runner" \
	--ocr-eng "$attachment_text_extraction_ocr_english" \
	--ocr-rus "$attachment_text_extraction_ocr_russian"
"$cargo_target_dir/debug/makosh-attachment-preview-assembly" \
	--build-id "$build_id" \
	--output-dir "$attachment_preview_assembly" \
	--runtime "$cargo_target_dir/debug/makosh-attachment-preview-runtime"
"$cargo_target_dir/debug/makosh-attachment-preview-evidence-replay-assembly" \
	--build-id "$build_id" \
	--output-dir "$attachment_preview_evidence_replay_assembly" \
	--runtime "$cargo_target_dir/debug/makosh-attachment-preview-evidence-replay-runtime"
"$cargo_target_dir/debug/makosh-attachment-translation-assembly" \
	--build-id "$build_id" \
	--output-dir "$attachment_translation_assembly" \
	--runtime "$cargo_target_dir/debug/makosh-attachment-translation-runtime"
"$cargo_target_dir/debug/makosh-ai-inference-assembly" \
	--build-id "$build_id" \
	--output-dir "$ai_inference_assembly" \
	--runtime "$cargo_target_dir/debug/makosh-ai-inference-runtime"
"$cargo_target_dir/debug/makosh-ollama-ai-assembly" \
	--build-id "$build_id" \
	--output-dir "$ollama_ai_assembly" \
	--runtime "$cargo_target_dir/debug/makosh-ollama-ai-runtime"
"$cargo_target_dir/debug/makosh-speech-to-text-assembly" \
	--build-id "$build_id" \
	--output-dir "$speech_to_text_assembly" \
	--runtime "$cargo_target_dir/debug/makosh-speech-to-text-runtime"
"$cargo_target_dir/debug/makosh-whisper-stt-assembly" \
	--output "$whisper_stt_assembly" \
	--build-id "$build_id" \
	--runtime "$cargo_target_dir/debug/makosh-whisper-stt-runtime" \
	--runner "$whisper_stt_runner" \
	--model "$whisper_stt_model"
"$cargo_target_dir/debug/makosh-calendar-assembly" \
	--build-id "$build_id" \
	--output-dir "$calendar_assembly" \
	--runtime "$cargo_target_dir/debug/makosh-calendar-runtime"
"$cargo_target_dir/debug/makosh-organizations-assembly" \
	--build-id "$build_id" \
	--output-dir "$organizations_assembly" \
	--runtime "$cargo_target_dir/debug/makosh-organizations-runtime"
"$cargo_target_dir/debug/makosh-documents-assembly" \
	--build-id "$build_id" \
	--output-dir "$documents_assembly" \
	--runtime "$cargo_target_dir/debug/makosh-documents-runtime"
"$cargo_target_dir/debug/makosh-relationships-assembly" \
	--build-id "$build_id" \
	--output-dir "$relationships_assembly" \
	--runtime "$cargo_target_dir/debug/makosh-relationships-runtime"
"$cargo_target_dir/debug/makosh-projects-assembly" \
	--build-id "$build_id" \
	--output-dir "$projects_assembly" \
	--runtime "$cargo_target_dir/debug/makosh-projects-runtime"
"$cargo_target_dir/debug/makosh-decisions-assembly" \
	--build-id "$build_id" \
	--output-dir "$decisions_assembly" \
	--runtime "$cargo_target_dir/debug/makosh-decisions-runtime"
"$cargo_target_dir/debug/makosh-search-assembly" \
	--build-id "$build_id" \
	--output-dir "$search_assembly" \
	--runtime "$cargo_target_dir/debug/makosh-search-runtime"
"$cargo_target_dir/debug/makosh-timeline-assembly" \
	--build-id "$build_id" \
	--output-dir "$timeline_assembly" \
	--runtime "$cargo_target_dir/debug/makosh-timeline-runtime"
"$cargo_target_dir/debug/makosh-graph-assembly" \
	--build-id "$build_id" \
	--output-dir "$graph_assembly" \
	--runtime "$cargo_target_dir/debug/makosh-graph-runtime"
"$cargo_target_dir/debug/makosh-memory-assembly" \
	--build-id "$build_id" \
	--output-dir "$memory_assembly" \
	--runtime "$cargo_target_dir/debug/makosh-memory-runtime"
"$cargo_target_dir/debug/makosh-consistency-assembly" \
	--build-id "$build_id" \
	--output-dir "$consistency_assembly" \
	--runtime "$cargo_target_dir/debug/makosh-consistency-runtime"
"$cargo_target_dir/debug/makosh-risk-assembly" \
	--build-id "$build_id" \
	--output-dir "$risk_assembly" \
	--runtime "$cargo_target_dir/debug/makosh-risk-runtime"
"$cargo_target_dir/debug/makosh-zoom-assembly" \
	--build-id "$build_id" \
	--output-dir "$zoom_assembly" \
	--runtime "$cargo_target_dir/debug/makosh-zoom-runtime"
"$cargo_target_dir/debug/makosh-telemost-assembly" \
	--build-id "$build_id" \
	--output-dir "$telemost_assembly" \
	--runtime "$cargo_target_dir/debug/makosh-telemost-runtime"
"$cargo_target_dir/debug/makosh-omniroute-assembly" \
	--build-id "$build_id" \
	--output-dir "$omniroute_assembly" \
	--runtime "$cargo_target_dir/debug/makosh-omniroute-runtime"
"$cargo_target_dir/debug/makosh-desktop-call-recording-assembly" \
	--build-id "$build_id" \
	--output-dir "$desktop_call_recording_assembly" \
	--runtime "$cargo_target_dir/debug/makosh-desktop-call-recording-runtime"
"$cargo_target_dir/debug/makosh-mail-assembly" \
	--build-id "$build_id" \
	--output-dir "$mail_assembly" \
	--runtime "$cargo_target_dir/debug/makosh-mail-runtime"
"$cargo_target_dir/debug/makosh-telegram-assembly" \
	--build-id "$build_id" \
	--output-dir "$telegram_assembly" \
	--runtime "$cargo_target_dir/debug/makosh-telegram-runtime" \
	--tdjson "$tdjson_path" \
	--tgcalls "$tgcalls_path"
"$cargo_target_dir/debug/makosh-whatsapp-assembly" \
	--build-id "$build_id" \
	--output-dir "$whatsapp_assembly" \
	--runtime "$cargo_target_dir/debug/makosh-whatsapp-runtime"
"$cargo_target_dir/debug/makosh-zulip-assembly" \
	--build-id "$build_id" \
	--output-dir "$zulip_assembly" \
	--runtime "$cargo_target_dir/debug/makosh-zulip-runtime"

base_input="$scratch_root/release-input.json"
node "$backend_root/scripts/build-local-platform-release-input.mjs" \
	--target "$target_triple" \
	--artifact-dir "$cargo_target_dir/debug" \
	--browser-bootstrap "$frontend_root/dist/index.html" \
	--browser-assets-dir "$frontend_root/dist/assets" \
	--output "$base_input" \
	--descriptor-dir "$scratch_root/descriptors" \
	--distribution-id "$distribution_id" \
	--generation "$distribution_generation" \
	--release-version "$release_version" \
	--build-id "$build_id" \
	--source-commit "$source_commit" \
	--lockfile-sha256 "$lockfile_sha256" \
	--sbom-sha256 "$sbom_sha256" \
	--toolchain-sha256 "$toolchain_sha256"

if test -e "$signing_key"; then
	require_regular_file "$signing_key" "development release signing key"
	test "$(stat -f '%Lp' "$signing_key")" = "600" \
		|| fail "development release signing key permissions must be 0600"
else
	node "$backend_root/scripts/generate-release-signing-key.mjs" --output "$signing_key"
	chmod 600 "$signing_key"
fi

node "$backend_root/scripts/build-distribution-release.mjs" \
	--input "$base_input" \
	--artifact-fragment "$communications_assembly/communications.release-artifacts.json" \
	--artifact-fragment "$communications_export_assembly/communications_export.release-artifacts.json" \
	--artifact-fragment "$communication_delivery_intent_assembly/communication_delivery_intent.release-artifacts.json" \
	--artifact-fragment "$communication_bulk_action_assembly/communication_bulk_action.release-artifacts.json" \
	--artifact-fragment "$communication_cross_channel_forward_assembly/communication_cross_channel_forward.release-artifacts.json" \
	--artifact-fragment "$communication_reply_suggestion_assembly/communication_reply_suggestion.release-artifacts.json" \
	--artifact-fragment "$communication_summary_assembly/communication_summary.release-artifacts.json" \
	--artifact-fragment "$communication_translation_assembly/communication_translation.release-artifacts.json" \
	--artifact-fragment "$communication_explanation_assembly/communication_explanation.release-artifacts.json" \
	--artifact-fragment "$communication_recipient_suggestion_assembly/communication_recipient_suggestion.release-artifacts.json" \
	--artifact-fragment "$communication_task_candidate_assembly/communication_task_candidate.release-artifacts.json" \
	--artifact-fragment "$communication_note_candidate_assembly/communication_note_candidate.release-artifacts.json" \
	--artifact-fragment "$review_task_candidate_assembly/review-task-candidate.release-artifacts.json" \
	--artifact-fragment "$tasks_assembly/tasks.release-artifacts.json" \
	--artifact-fragment "$review_obligation_candidate_assembly/review-obligation-candidate.release-artifacts.json" \
	--artifact-fragment "$obligations_assembly/obligations.release-artifacts.json" \
	--artifact-fragment "$reviewed_obligation_candidate_promotion_assembly/reviewed_obligation_candidate_promotion.release-artifacts.json" \
	--artifact-fragment "$persons_assembly/persons.release-artifacts.json" \
	--artifact-fragment "$identity_resolution_assembly/identity-resolution.release-artifacts.json" \
	--artifact-fragment "$mail_persons_sync_assembly/mail_persons_sync.release-artifacts.json" \
	--artifact-fragment "$review_person_match_candidate_assembly/review-person-match-candidate.release-artifacts.json" \
	--artifact-fragment "$reviewed_person_match_candidate_promotion_assembly/reviewed-person-match-candidate-promotion.release-artifacts.json" \
	--artifact-fragment "$knowledge_assembly/knowledge.release-artifacts.json" \
	--artifact-fragment "$review_note_candidate_assembly/review-note-candidate.release-artifacts.json" \
	--artifact-fragment "$reviewed_note_candidate_promotion_assembly/reviewed_note_candidate_promotion.release-artifacts.json" \
	--artifact-fragment "$reviewed_task_candidate_promotion_assembly/reviewed_task_candidate_promotion.release-artifacts.json" \
	--artifact-fragment "$communication_delayed_delivery_assembly/communication_delayed_delivery.release-artifacts.json" \
	--artifact-fragment "$attachment_security_assembly/attachment-security.release-artifacts.json" \
	--artifact-fragment "$attachment_text_extraction_assembly/attachment_text_extraction.release-artifacts.json" \
	--artifact-fragment "$attachment_preview_assembly/attachment-preview.release-artifacts.json" \
	--artifact-fragment "$attachment_preview_evidence_replay_assembly/attachment_preview_evidence_replay.release-artifacts.json" \
	--artifact-fragment "$attachment_translation_assembly/attachment_translation.release-artifacts.json" \
	--artifact-fragment "$ai_inference_assembly/ai-inference.release-artifacts.json" \
	--artifact-fragment "$ollama_ai_assembly/ollama-ai.release-artifacts.json" \
	--artifact-fragment "$speech_to_text_assembly/speech-to-text.release-artifacts.json" \
	--artifact-fragment "$whisper_stt_assembly/whisper-stt.release-artifacts.json" \
	--artifact-fragment "$calendar_assembly/calendar.release-artifacts.json" \
	--artifact-fragment "$organizations_assembly/organizations.release-artifacts.json" \
	--artifact-fragment "$documents_assembly/documents.release-artifacts.json" \
	--artifact-fragment "$relationships_assembly/relationships.release-artifacts.json" \
	--artifact-fragment "$projects_assembly/projects.release-artifacts.json" \
	--artifact-fragment "$decisions_assembly/decisions.release-artifacts.json" \
	--artifact-fragment "$search_assembly/search.release-artifacts.json" \
	--artifact-fragment "$timeline_assembly/timeline.release-artifacts.json" \
	--artifact-fragment "$graph_assembly/graph.release-artifacts.json" \
	--artifact-fragment "$memory_assembly/memory.release-artifacts.json" \
	--artifact-fragment "$consistency_assembly/consistency.release-artifacts.json" \
	--artifact-fragment "$risk_assembly/risk.release-artifacts.json" \
	--artifact-fragment "$zoom_assembly/zoom.release-artifacts.json" \
	--artifact-fragment "$telemost_assembly/telemost.release-artifacts.json" \
	--artifact-fragment "$omniroute_assembly/omniroute.release-artifacts.json" \
	--artifact-fragment "$desktop_call_recording_assembly/desktop-call-recording.release-artifacts.json" \
	--artifact-fragment "$mail_assembly/mail.release-artifacts.json" \
	--artifact-fragment "$telegram_assembly/telegram.release-artifacts.json" \
	--artifact-fragment "$whatsapp_assembly/whatsapp.release-artifacts.json" \
	--artifact-fragment "$zulip_assembly/zulip.release-artifacts.json" \
	--signing-key "$signing_key" \
	--trust-root "$resource_root/makosh-release-trust-root.pb" \
	--signed-manifest "$resource_root/makosh-signed-distribution-manifest.pb" \
	--distribution-root "$resource_root/distribution"

cp "$cargo_target_dir/debug/makosh-kernel" "$app_root/Contents/MacOS/makosh-kernel"
chmod 700 "$app_root/Contents/MacOS/makosh-kernel"
printf '%s\n' "$distribution_generation" \
	>"$new_release_root/$generation_metadata_name"
chmod 600 "$new_release_root/$generation_metadata_name"

previous_release_root="$local_root/dev-release-previous.$$"
case "$release_root" in
	"$local_root"/*) ;;
	*) fail "development release root must remain inside the project-local state directory" ;;
esac
if test -e "$previous_release_root"; then
	fail "temporary previous release path already exists"
fi
if test -e "$release_root"; then
	mv "$release_root" "$previous_release_root"
fi
mv "$new_release_root" "$release_root"
if test -e "$previous_release_root"; then
	rm -rf -- "$previous_release_root"
fi

kernel_path="$release_root/МакошьDev.app/Contents/MacOS/makosh-kernel"
require_regular_file "$kernel_path" "materialized development Kernel"
require_regular_file \
	"$release_root/$generation_metadata_name" \
	"materialized development release generation metadata"
printf 'development-release: ready distribution=%s generation=%s\n' \
	"$distribution_id" "$distribution_generation" >&2
printf '%s\n' "$kernel_path"
