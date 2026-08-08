# ADR-0391: Whisper STT provider integration

Статус: Принято

Дата: 2026-08-04

Состояние реализации: реализовано. `whisper_stt_provider_v1` имеет отдельные
contract/core/process/persistence/runtime/assembly units, pinned native release
и model bytes, signed managed admission, real-audio conformance и restart/replay
evidence. `speech_to_text_engine_v1` реализован отдельно и не импортирует
Whisper implementation.

Уточняет:

- [ADR-0204: Bundled integration plugins](ADR-0204-bundled-integration-plugins-and-provider-neutral-context-boundary.md);
- [ADR-0212: Crate topology and compile isolation](ADR-0212-crate-topology-and-compile-isolation.md);
- [ADR-0219: Managed module distribution integrity](ADR-0219-managed-module-distribution-integrity-and-explicit-updates.md);
- [ADR-0230: Blob platform opaque references](ADR-0230-blob-platform-opaque-references-and-owner-local-metadata.md);
- [ADR-0390: Call recording custody and Speech-to-Text boundary](ADR-0390-call-recording-custody-and-speech-to-text-boundary.md).

## Контекст

Speech-to-Text engine определяет provider-neutral request/result и выбирает
provider только по exact contract. Для production gate нужен concrete local
provider. Legacy external command и fixture transcript являются behavior
evidence, но не дают pinned executable/model, managed custody, idempotency,
privacy или restart guarantees.

Whisper не является Communications domain, transcription workflow или частью
STT engine. Provider-specific executable flags, model format, temporary files и
diagnostics не должны попадать в engine contract, Kernel или frontend.

## Решение

`whisper_stt` является отдельной bundled integration с owner
`whisper_stt`. Она предоставляет exact
`speech_to_text.provider_transcribe.v1` request RPC из
`makosh-speech-to-text-api` и не предоставляет business/client API.

```text
Speech-to-Text engine
  -> capability-routed provider request
  -> Whisper integration runtime
  -> receipt-bound Blob audio read
  -> pinned whisper.cpp executable + pinned model
  -> canonical transcript artifact
  -> target-bound Blob write to engine
  -> typed provider result
```

Integration не импортирует Communications или call-transcription contracts.
STT engine не импортирует Whisper packages и не выбирает provider по имени.

### Build units

- `makosh-speech-transcript-artifact` — canonical private Blob document schema
  и bounded validation; contract не является query/SSE/event payload;
- `makosh-whisper-stt-core` — provider execution policy, input/result plan и
  idempotency invariants без process/Blob/storage;
- `makosh-whisper-stt-process` — exact whisper.cpp CLI dialect, private work
  files, bounded timeout/output parsing и no-shell execution;
- `makosh-whisper-stt-persistence` — integration-local idempotency/fences and
  safe terminal metadata without audio, transcript or custody proofs;
- `makosh-whisper-stt-runtime` — managed request handler, Blob read/write,
  process orchestration and settings application;
- `makosh-whisper-stt-assembly` — unsigned runtime, descriptor, settings,
  storage, native executable and model artifacts.

### Canonical transcript artifact

Transcript bytes используют deterministic Protobuf
`SpeechTranscriptDocumentV1` из отдельного contract unit. Document содержит
protocol major, request ID, detected language, ordered bounded segments and
segment UTF-8 content. Он не содержит provider/model names, filesystem paths,
credentials, debug output, arbitrary maps или custody proof.

Document никогда не входит в request/result RPC, durable envelope, PostgreSQL,
health, logs или SSE. Provider записывает exact encoded bytes в Blob с target,
который Kernel вывел из authenticated STT engine capability. Engine получает
только reference/size/hash/proof и execution receipt.

### Native release and settings

Runtime получает два exact managed artifacts:

- `whisper_stt.model.v1` с use `read_only_data`;
- `whisper_stt.runner.v1` с use `native_executable`.

Artifacts связаны с `makosh-whisper-stt-runtime`, входят в signed distribution
manifest, копируются Kernel в private staged root и повторно проверяются runtime
по size, SHA-256, inode и mode. System executable/model fallback, PATH lookup,
runtime download, symlink и mutable shared model запрещены.

Settings содержат только bounded execution policy: language policy, thread
budget, timeout and output limits. Executable/model paths и hashes не являются
Settings; их authority — signed release artifacts. Audio/transcript и custody
proof не являются Settings или Vault values.

### Process and privacy

Process запускается без shell и environment inheritance, в private work
directory, с explicit arguments and closed stdin. Audio materialизуется в
bounded private file только на время invocation; transcript output читается с
hard limit, валидируется и кодируется в canonical artifact, затем work root
удаляется. Stdout/stderr не возвращаются и не логируются; наружу выходит только
fixed reject code.

Provider persistence хранит request/digest, source hash, model revision hash,
settings/policy revisions, terminal status и transcript reference/hash/size.
Audio bytes, transcript text/segments, source or target custody proofs,
filesystem paths, executable output and provider-private diagnostics запрещены.

Replay повторяет safe materialization/execution при необходимости, сверяет
terminal metadata и получает новый Blob proof. Changed result under the same
request/digest is conflict. Ambiguous process or Blob write outcome не
автоматически создаёт новый artifact identity без reconciliation.

## SRP и запрещённые зависимости

- artifact contract знает только canonical private document;
- core знает только provider execution semantics;
- process знает только exact native dialect and private files;
- persistence знает только safe durable metadata;
- runtime координирует public ports;
- assembly только материализует unsigned release inputs.

Запрещены:

- Whisper integration importing Communications/call-transcription/domain
  implementation;
- STT engine importing Whisper implementation;
- direct module socket/store/SQL;
- provider/model/path fields в STT public request/result;
- transcript/audio through PostgreSQL, events, query, SSE, logs or health;
- system executable/model fallback or runtime download;
- fixture transcript как production evidence.

## Проверка

Gate открывается только когда:

1. все build units имеют compile/SRP isolation и negative dependency guards;
2. transcript artifact schema/validator доказывает bounds, ordering and UTF-8;
3. native build pins exact whisper.cpp source/model digests and is reproducible;
4. process tests cover invalid audio, malformed/oversized output, timeout,
   non-zero exit, symlink/path substitution and cleanup;
5. disposable PostgreSQL proves idempotency, conflict and restart state without
   private content;
6. signed managed contour launches separate engine/provider processes through
   Storage/Vault/Blob/Core routing;
7. bounded real audio fixture produces exact transcript artifact and fresh
   target-bound proof;
8. duplicate/restart/provider outage/Blob outage/revoke/grant and storage
   generation changes fail closed or retry safely;
9. privacy scan proves no audio/transcript/path/stdout/stderr/proof leakage;
10. full `make pre-push` passes before both provider and dependent STT engine
    inventory gates are changed to `implemented`.

## Последствия

Whisper становится replaceable integration, а не условной веткой в engine.
Release становится тяжелее из-за pinned executable/model artifacts, но runtime
не зависит от host installation и production evidence воспроизводимо.
