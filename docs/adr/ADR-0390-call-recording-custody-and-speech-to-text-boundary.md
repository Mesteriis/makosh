# ADR-0390: Call recording custody and speech-to-text boundary

Статус: Принято

Дата: 2026-08-03

Состояние реализации: реализовано; `call_transcription_v1` имеет состояние
`implemented`. Workflow имеет exact ingress, generated client API, pure
lifecycle core, owner-local persistence, signed managed runtime/release,
atomic inbox/outbox, fenced jobs/recovery, exact recording event subscriptions,
public STT request RPC, metadata-only replayable SSE, actor/session-bound
one-use Blob tickets и replay-safe source custody без proof persistence.
Live conformance доказывает real recording WAV, NATS outage/reconnect,
Speech-to-Text/Whisper, exact transcript ClientBlob, wrong actor, Blob outage и
workflow restart. Полный root `make pre-push` является финальным evidence gate.
Компилируемый workflow, который передаёт текст Communications в LLM или
возвращает summary вместо transcript, не является реализацией этого ADR.

По состоянию на 2026-08-04 `desktop_call_recording_v1`,
`speech_to_text_engine_v1` и `whisper_stt_provider_v1` реализованы и допущены
отдельно. Доказаны explicit native consent, exact managed request RPC,
provider-neutral Blob custody chain, owner-local PostgreSQL, pinned native/model
resources, signed managed admission, real-audio conformance и restart/replay.
Эти independent gates сами по себе не открывают `call_transcription_v1`.

Уточняет:

- [ADR-0201: Core module communication and NATS](ADR-0201-core-module-communication-and-nats.md);
- [ADR-0204: Bundled integration plugins and provider-neutral context](ADR-0204-bundled-integration-plugins-and-provider-neutral-context-boundary.md);
- [ADR-0212: Crate topology and compile isolation](ADR-0212-crate-topology-and-compile-isolation.md);
- [ADR-0213: Code ownership and module autonomy](ADR-0213-code-ownership-and-module-autonomy.md);
- [ADR-0220: Canonical durable envelope](ADR-0220-canonical-durable-envelope-and-contract-evolution.md);
- [ADR-0230: Blob platform opaque references](ADR-0230-blob-platform-opaque-references-and-owner-local-metadata.md);
- [ADR-0282: Full Communications capability reconstruction](ADR-0282-full-communications-and-settings-capability-reconstruction.md);
- [ADR-0284: Telegram one-to-one audio calls](ADR-0284-telegram-one-to-one-audio-calls-operational-boundary.md);
- [ADR-0349: Event-backed Communications call evidence](ADR-0349-event-backed-communications-call-evidence.md).

## Контекст

Legacy имел две разные поверхности: fixture STT для Telegram call rows и
настраиваемый внешний transcriber для локально записанного conference bundle.
Обе являются только behavior evidence. Fixture не доказывает production
transcription, а внешний command смешивал workflow, provider execution,
filesystem paths и projection в другие domains.

Clean-room Communications call evidence намеренно не содержит audio, media
keys, provider locators, recording references или transcript. Telegram call
media adapter обеспечивает разговор и отдельно запрещает hidden recording.
Следовательно, Communications runtime не может подготовить transcript source,
а generic AI/LLM inference не является speech-to-text provider.

Прототип, использующий `source_message_id`, `sender/subject/body`,
`summary_utf8` или `CallTranscriptionLength`, функционально является копией
communication summary. Его нельзя открывать как `call_transcription_v1`, даже
если Cargo и unit tests проходят.

## Решение

`call_transcription` является отдельным workflow owner. Он не является
Communications domain, provider integration, AI engine или recording service.

```text
explicit owner-authorized recording source
  -> source-owned Blob custody and durable recording-ready event
  -> call_transcription owner-local inbox/join
  -> speech_to_text request_rpc
  -> exact STT provider integration
  -> transcript artifact in call_transcription-owned Blob custody
  -> metadata query + actor-bound one-use client_blob ticket + SSE status
```

Workflow принимает только завершённую owner-authorized recording. Он не
включает микрофон, не записывает звонок скрыто, не читает provider runtime и не
пытается получить audio из Communications call evidence.

### Functional units

`call_transcription_v1` состоит из независимых units с одной причиной
изменения:

- `makosh-call-transcription-api` — generated Start/Get/ReadTranscript и
  realtime client contracts;
- `makosh-call-transcription-ingress` — target-owned recording-ready/rejected
  durable contracts и target Blob tuple;
- `makosh-call-transcription-core` — provider-neutral lifecycle и invariants;
- `makosh-call-transcription-persistence` — owner-local inbox, runs, jobs,
  outbox, tickets and recovery;
- `makosh-call-transcription-runtime` — managed orchestration, event consumer,
  STT request routing, Blob materialization и client ports;
- `makosh-call-transcription-assembly` — unsigned runtime, descriptor,
  settings schema и storage artifacts.

Speech recognition является отдельным engine:

- `makosh-speech-to-text-api` — typed request/result contract;
- `makosh-speech-to-text-core` — engine lifecycle and result validation;
- `makosh-speech-to-text-persistence` — engine-local durable execution state;
- `makosh-speech-to-text-runtime` — request routing and provider selection by
  admitted capability, without provider-name branches;
- `makosh-speech-to-text-assembly` — exact engine release artifacts.

Concrete Whisper/whisper.cpp execution является integration и получает
собственные contract/core/process/runtime/assembly units. Она не импортируется
в workflow или engine implementation. Другой provider может реализовать тот же
public STT provider contract отдельным admission slice.

Recording acquisition также не входит в transcription workflow. Первый
producer обязан иметь отдельный owner, explicit UI/OS consent и source-owned
Blob custody. Provider-owned cloud recording, desktop capture и imported audio
являются разными producer slices и не маскируются одним generic adapter.

### Client contract

`StartCallTranscriptionRequestV1` содержит:

- stable `operation_id`;
- canonical `call_evidence_id` and exact expected revision;
- recording evidence ID and exact recording revision;
- owner-consent receipt ID and policy revision;
- requested language (`auto`, `en`, `ru`, `es` in v1).

Клиент не передаёт provider identity, filesystem path, Blob reference, model,
prompt, executable path, audio bytes или arbitrary metadata. Поля `length` и
`summary` отсутствуют: transcription возвращает transcript, а summarization
остаётся отдельным downstream workflow.

`Get` возвращает lifecycle, revisions, detected language, duration, segment
count, confidence/completeness and artifact availability. Transcript text и
speaker segments не хранятся в PostgreSQL и не передаются через Get или SSE.
`ReadTranscript` выдаёт actor-bound one-use ticket и затем exact Blob bytes.

### Recording source and consent

`makosh-call-transcription-ingress` определяет exact recording-ready payload:

- request/run/call evidence identities and revisions;
- recording evidence ID and revision;
- consent receipt ID, consent policy revision and consent scope;
- canonical audio media type, declared bytes, duration and SHA-256;
- target-bound Blob reference and custody transfer proof;
- logical owner ID.

V1 принимает только явно объявленный bounded canonical audio format. Source
producer отвечает за decode/normalization и обязан fail closed до публикации,
если формат, duration, size, consent или custody не доказаны. Durable payload
не содержит audio, filesystem path, provider locator, participant identity,
media encryption material или credentials.

Consent является authority receipt, а не boolean setting. Receipt связывает
actor/device, call evidence, recording evidence, purpose
`call_transcription`, policy revision and bounded capture interval. Revoke,
owner-device fence или stale revision запрещает новую transcription authority;
terminal artifact retention управляется отдельной owner policy.

### STT engine and provider boundary

Workflow вызывает только exact `speech_to_text.transcribe.v1` request RPC.
Запрос содержит target-bound audio Blob receipt, format/duration, language,
limits, consent binding and correlation. Engine не читает workflow storage.

Engine маршрутизирует запрос к admitted STT provider contract. Provider
integration владеет executable/model configuration, model digest, decoding,
timeouts and provider errors. Executable запускается без shell, по exact
absolute path and pinned binary/model hashes, с bounded stdin/stdout/files and
deadline. Settings и Vault не содержат audio или transcript.

Engine не знает owner/module identity конкретного provider. Для передачи Blob
custody Kernel атомарно разрешает тот же exact request-provider contract,
проверяет dependency/grants/current runtime fences и выпускает proof на
фактически выбранные provider owner/module/capability. Caller не передаёт эти
координаты в provider-neutral режиме. Явная target delegation остаётся отдельным
низкоуровневым режимом и не используется STT engine для выбора Whisper.

Blob-результат request RPC также не требует business dependency от provider к
caller. Authenticated caller может передать только ID собственной текущей
granted Blob capability как `response_blob_capability_id`. Kernel сверяет её с
registration, module и grant epoch вызывающего runtime и сам добавляет в
provider delivery exact transport-only owner/module/capability target. Caller
не может передать owner/module coordinates, provider не выбирает arbitrary
target и не импортирует contract workflow/domain, которому принадлежит
результат. Empty capability означает, что Blob-результат этим request RPC не
запрашивается.

Generic `ai.inference`, Ollama text generation и Communications AI source
contracts не участвуют в этом flow. Последующий summary transcript является
отдельным explicit workflow над transcript artifact и не расширяет
`call_transcription_v1`.

### Lifecycle and durable state

Core lifecycle:

```text
accepted
  -> awaiting_recording
  -> awaiting_stt
  -> materializing_transcript
  -> ready | rejected
```

Terminal state immutable. Every transition increments exact revision. Same
operation/fingerprint is idempotent; changed call, recording, consent,
language or policy under the same operation is conflict.

Persistence may store only identities, revisions, hashes, bounded metadata,
fences, exact STT request/result receipts, cleanup authority and outbox bytes.
Raw audio, transcript text, speaker text, provider debug output, executable
stdout/stderr and Blob custody secrets are forbidden in PostgreSQL, logs,
errors, health and SSE.

Поскольку custody proof не сохраняется, replay готового request RPC не строит
fake proof из PostgreSQL. Engine повторно вызывает idempotent provider с новым
custody delegation, проверяет, что reference/hash/metadata совпадают с durable
terminal record, и только затем делегирует новый proof вызывающему workflow.
Несовпадение является conflict; provider/Blob outage оставляет replay
retryable.

Inbox ID/hash is committed atomically with transition/outbox. ACK occurs only
after commit. Restart recovery revalidates current runtime/storage/grant,
consent, source revision and Blob custody; stale provider result or previous
generation authority is rejected.

## Dependency gates

Inventory получает отдельные prerequisites:

```text
call_recording_source_v1
speech_to_text_engine_v1
whisper_stt_provider_v1
        \ | /
communications_call_evidence_v1 + blob_v1
        \ | /
call_transcription_v1
```

`call_transcription_v1` не переводится в `implemented` по наличию packages или
fixture provider. Production admission каждого owner выполняется exact phase
gate; workflow, engine and integration остаются разными owners и build units.

## SRP and forbidden dependencies

- Communications owns canonical call evidence only.
- Recording producer owns capture, consent acquisition and source custody.
- `call_transcription` owns orchestration and transcript artifact lifecycle.
- `speech_to_text` owns provider-neutral recognition execution policy.
- Whisper integration owns concrete model/process behavior.
- Blob owns bytes and custody transfer; Kernel routes authority but does not
  inspect audio/transcript.
- App composes admitted surfaces and never becomes backend recording authority.

Forbidden:

- Communications importing transcription/STT/provider implementation;
- workflow importing Communications, recording producer, STT engine or Whisper
  implementation;
- engine importing provider integration implementation;
- integration importing Communications or transcription workflow;
- direct module socket/store/SQL access;
- provider-name switch in domain/workflow/engine core;
- transcript or audio through durable event, client query, SSE or logs;
- fixture STT as production evidence.

Successful transcript reads return the exact STT-owned
`SpeechTranscriptDocumentV1` protobuf bytes through the actor/session-bound
ClientBlob route. The app decodes that generated artifact contract and never
interprets the Blob as an untyped UTF-8 response.

## Проверка

Gate opens only after all evidence below is current:

1. architecture/SRP/Cargo isolation covers every declared unit and forbids the
   dependencies above;
2. generated ingress and client descriptors have exact schema hashes and
   privacy-negative assertions;
3. core covers idempotency conflict, consent/source revision conflict,
   terminal immutability and bounded metadata;
4. disposable PostgreSQL proves migrations, inbox/hash replay, atomic outbox,
   job lease recovery, tickets and cleanup;
5. recording producer proves explicit consent, bounded canonical audio and
   target-bound Blob custody without hidden capture;
6. real admitted STT provider transcribes a bounded audio fixture with pinned
   executable and model bytes; missing/mismatched binary/model, malformed
   audio, timeout and oversized output fail closed;
7. managed contour starts separate producer, workflow, STT engine and provider
   processes through Vault/Storage/Blob/NATS and Core capability routing;
8. authenticated Start/Get/ReadTranscript and pre-opened replayable SSE reach
   terminal state without polling; wrong actor, ticket replay and stale client
   session fail closed;
9. duplicate, conflicting replay, NATS outage, Blob outage, provider outage,
   runtime restart, grant/storage generation change, revoke and stale consent
   are covered;
10. privacy scan proves no audio, transcript, paths, provider IDs, secrets or
    custody proof in PostgreSQL, envelopes, logs, errors, health or SSE;
11. full `make pre-push` passes before reconstruction inventory changes to
    `implemented`.

## Последствия

Полный перенос требует больше одного workflow crate: recording acquisition,
STT engine and provider integration становятся явными units. Это увеличивает
inventory, но устраняет ложную реализацию через Communications text/LLM и
сохраняет domain/integration/workflow границы. Frontend может показывать
skeleton до admission, но backend не публикует fake transcript или active
control.
