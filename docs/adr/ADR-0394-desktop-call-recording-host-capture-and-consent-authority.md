# ADR-0394: Desktop call recording host capture and consent authority

Статус: Принято

Дата: 2026-08-04

Состояние реализации: implemented; gate `desktop_call_recording_v1` открыт.
Пять integration packages, target-owned ingress foundation и Tauri host adapter
прошли managed, PostgreSQL, authenticated client, Blob/NATS outage и restart
проверки из раздела «Проверка». Это не открывает отдельный
`call_transcription_v1` workflow gate.

Уточняет:

- [ADR-0204: Bundled integration plugins and provider-neutral context](ADR-0204-bundled-integration-plugins-and-provider-neutral-context-boundary.md);
- [ADR-0205: Core Gateway and client transport](ADR-0205-core-gateway-and-client-transport.md);
- [ADR-0218: Owner device identity](ADR-0218-owner-device-identity-enrollment-and-offline-recovery.md);
- [ADR-0220: Canonical durable envelope](ADR-0220-canonical-durable-envelope-and-contract-evolution.md);
- [ADR-0230: Blob platform opaque references](ADR-0230-blob-platform-opaque-references-and-owner-local-metadata.md);
- [ADR-0337: Capability-routed managed client realtime](ADR-0337-capability-routed-managed-client-realtime.md);
- [ADR-0390: Call recording custody and Speech-to-Text boundary](ADR-0390-call-recording-custody-and-speech-to-text-boundary.md).

## Контекст

Для первого production producer записи звонка нужен доступ к desktop audio
input, который выдаёт OS только host application. Managed module не должен
обходить системный permission UI, а Tauri host не должен становиться business
owner, хранить durable recording truth или публиковать business event.

Активный Telemost companion содержит historical implementation evidence:
`consent_attested: bool`, произвольный путь к `ffmpeg`, MP3 и возвращаемые
filesystem paths. Это не authority receipt и не clean-room custody. Данный код
не является реализацией `desktop_call_recording_v1` и подлежит удалению после
появления нового host adapter.

## Решение

`desktop_call_recording` является отдельной bundled integration. Она не
является Communications domain, transcription workflow, Blob platform или
frontend service. Единственный v1 source — явно выбранный пользователем
desktop audio input; provider cloud recordings и imported files требуют
отдельных producer gates.

```text
authenticated visible client action
  -> desktop_call_recording client command through Core Gateway
  -> owner-local pending consent challenge
  -> native Tauri sheet + OS audio permission
  -> fenced private host bridge capture
  -> integration-owned Blob write
  -> target-bound custody delegation
  -> call_transcription-owned recording-ready durable event
```

Kernel и Core Gateway маршрутизируют transport authority. Они не определяют
текст consent, не выбирают audio device, не получают audio bytes и не
интерпретируют recording metadata.

### Functional units

Production slice разделён по причинам изменения:

- `makosh-desktop-call-recording-api` — generated client, realtime и private
  host-bridge contracts;
- `makosh-desktop-call-recording-core` — provider-neutral capture lifecycle,
  consent binding, canonical audio validation и idempotency;
- `makosh-desktop-call-recording-persistence` — owner-local challenges,
  sessions, host command leases, inbox/outbox и replay transitions;
- `makosh-desktop-call-recording-runtime` — managed client/host ports, Blob
  materialization, custody delegation и durable event publication;
- `makosh-desktop-call-recording-assembly` — unsigned runtime, descriptor,
  settings schema и storage artifacts;
- Tauri `desktop_call_recording_host` adapter — visible native permission and
  bounded capture only. Он зависит от public recording contract и runtime
  protocol, но не от Communications, transcription, Blob implementation или
  recording runtime implementation.

Target-owned durable event остаётся в отдельной
`makosh-call-transcription-ingress` unit. Source integration может зависеть
только от этого exact public ingress contract; workflow implementation не
импортируется.

### Consent authority

`StartDesktopCallRecordingRequestV1` не содержит boolean `consent_attested`.
Authenticated request связывает stable operation, canonical call evidence ID
и expected revision, requested language-independent capture purpose и bounded
maximum duration. Runtime создаёт одноразовый challenge для exact logical
owner and authenticated device.

Tauri показывает native sheet с неизменяемым purpose
`call_transcription`, call anchor, выбранным audio input и maximum duration.
Только отдельное affirmative native action и успешный current OS permission
разрешают host начать capture. Cancel, denied OS permission, expired challenge,
wrong device, reused challenge, runtime/grant generation change or revoke fail
closed.

Terminal consent receipt создаёт integration runtime после host completion. Он
связывает:

- logical owner and authenticated device actor digest;
- call evidence ID/revision;
- recording evidence ID/revision;
- purpose `call_transcription` and consent policy revision;
- exact capture start/end timestamps and maximum duration;
- host route binding, runtime generation and grant epoch.

Receipt ID публикуется, но receipt body/signature не покидают owner-local
integration storage. Boolean, frontend state, OS permission alone или наличие
аудиофайла не являются consent authority.

### Host bridge and audio custody

Kernel выдаёт стандартный short-lived
`ManagedIntegrationHostBridgeConfigurationV1`. Handshake проверяет exact route
binding. Host claims only commands for the current local runtime and submits
only typed lifecycle operations:

- `accept_consent_and_begin`;
- bounded canonical audio completion;
- `capture_rejected`.

V1 canonical format — RIFF/WAVE, signed little-endian PCM, mono, 16 kHz,
16-bit. Host производит эти bytes напрямую; external executable path, shell,
provider SDK, MP3 and arbitrary codec configuration запрещены. Runtime повторно
проверяет container/header, declared size, duration and SHA-256 before Blob IO.

Audio допускается только в bounded private host-bridge request и одноразовой
Blob write session текущей integration runtime. Оно запрещено в durable
envelope, PostgreSQL, client response, SSE, logs, errors and health. Maximum v1
audio body — 64 MiB; capture duration additionally bounded by accepted
challenge. Runtime не сохраняет filesystem path или staging path.

После успешной source-owned Blob write integration запрашивает target-bound
custody delegation к exact `call_transcription` recording ingress capability.
Только затем она атомарно сохраняет terminal metadata и recording-ready outbox.
Custody proof не декодируется и не нормализуется в отдельные PostgreSQL columns:
до публикации он существует только внутри exact opaque outbox envelope bytes,
которые ADR-0220 требует сохранять без re-encode. Query, projection, log и
client surfaces proof не получают. Failure публикует typed bodyless rejection
без audio/path/proof.

### Client surface and realtime

Client operational surface предоставляет generated `Start`, `Stop` and `Get`.
Start возвращает receipt accepted/pending, но не означает, что OS capture
начался. Stop является idempotent intent; completion приходит через один
существующий replayable Gateway SSE stream. Get разрешён как initial/manual
recovery snapshot, periodic polling запрещён.

Client response/SSE содержит только operation/recording IDs, sanitized state,
revision, bounded duration and public error code. ADR-0396 дополнительно
разрешает только terminal `Get` выдавать typed opaque transcription authority;
SSE её не содержит. Audio, Blob reference/proof, consent body, device identity,
audio input label and filesystem path запрещены.

## SRP и запрещённые зависимости

- Communications owns only canonical call evidence.
- `desktop_call_recording` owns consent receipt, capture lifecycle and source
  Blob custody.
- `call_transcription` owns recording ingress and transcript orchestration.
- Tauri owns native prompt/OS capture mechanics only.
- Blob owns encrypted bytes; Kernel owns routing and fences only.

Запрещены:

- Communications importing recording/transcription code;
- recording integration importing Communications or workflow implementation;
- workflow importing recording implementation;
- Tauri publishing durable events or writing business PostgreSQL;
- direct module socket/store/SQL outside the admitted host bridge;
- provider-name branches or Telemost-specific fields in recording contracts;
- boolean consent, hidden/autostart capture, raw paths or external executable
  configuration;
- audio в PostgreSQL/events/client query/SSE/logs/errors;
- decoded/normalized custody proof в PostgreSQL и любой custody proof в client
  query/SSE/logs/errors; exact opaque durable outbox bytes являются единственным
  временным исключением до подтверждённой broker publication.

## Проверка

Текущий implementation evidence:

- Tauri adapter разделён на route-bound transport, native consent и bounded
  CoreAudio capture responsibilities; он собирается только с exact feature
  `desktop-call-recording-host`;
- macOS bundle объявляет microphone usage purpose и audio-input entitlement;
- native sheet показывает immutable purpose, call anchor/revision, выбранный
  system audio input и maximum duration до попытки открыть OS capture;
- capture непосредственно формирует bounded mono 16 kHz PCM/WAVE без shell и
  внешнего executable;
- legacy Telemost recorder, MP3/path receipts, boolean consent и speaker-file
  relay удалены, а provider WebView не получил recording authority;
- unit и architecture evidence покрывают no-autostart, prompt binding,
  cancel/permission-denied branches, canonical WAV bounds, private route
  binding и запрещённые зависимости;
- Tauri main window не получает recording host permissions из static bundle:
  exact connect/disconnect capability добавляется только после проверки
  owner-private Kernel route descriptor, а connect повторно валидирует current
  descriptor и binding;
- disposable PostgreSQL доказывает idempotent lifecycle, command leases,
  malformed/truncated WAV rejection, atomic terminal state, outbox и replay;
- signed managed contour запускает отдельные Recording, Storage, Vault, Blob и
  NATS runtimes и проводит bounded canonical WAV через source Blob write и
  target-owned custody/event ingress;
- authenticated Gateway Start/Stop/Get и заранее открытый shared SSE достигают
  terminal state без polling; wrong session и replay gap fail closed;
- NATS outage сохраняет terminal state и exact durable envelope до relay, Blob
  outage даёт typed rejection без остановки Recording runtime, а successor
  restart отвергает stale host route;
- полный `make pre-push` прошёл перед изменением reconstruction inventory на
  `implemented`.

Gate `desktop_call_recording_v1` открывается только когда:

1. все пять production packages и Tauri host adapter имеют exact compile
   isolation и production inventory;
2. generated client/host/realtime and target-owned ingress descriptors имеют
   pinned hashes и privacy-negative assertions;
3. core покрывает idempotency/conflict, one-use/expiry/wrong-device consent,
   stale call/runtime/grant revision, terminal immutability and WAV bounds;
4. disposable PostgreSQL доказывает challenges, command lease recovery,
   transitions, atomic outbox and restart replay;
5. managed contour запускает отдельную integration через signed admission,
   Storage, Blob, NATS and host bridge;
6. native host test доказывает visible affirmative action, denied permission,
   cancel and no hidden/autostart capture;
7. bounded real WAV проходит source Blob write, target custody and exact
   recording-ready event; malformed/oversized/truncated audio and Blob/NATS
   outage fail closed;
8. authenticated Start/Stop/Get and pre-opened shared SSE reach terminal state
   without polling; wrong actor/session and replay gap fail closed;
9. privacy scan доказывает отсутствие audio/path/input label/consent body,
   device identity and custody proof в persistent/public surfaces;
10. legacy Telemost recorder commands and `consent_attested` surface удалены;
11. полный `make pre-push` проходит до изменения reconstruction inventory на
    `implemented`.

## Последствия

Host остаётся необходимым trusted adapter для OS capture, но не превращается в
domain или integration facade. Recording producer получает собственные units,
authority and storage; Communications и transcription видят только свои exact
public contracts. Skeleton UI допустим до admission, active/fake recording —
нет.
