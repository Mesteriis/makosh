# ADR-0353: Communication reply suggestion and AI inference boundary

Статус: Принято

Дата: 2026-07-30

Состояние реализации: architecture agreement, Communications-owned source
contract/runtime handoff и public AI contract unit `makosh-ai-contracts`
реализованы. Communications имеет durable command/results, inbox/hash fencing,
current-revision validation, target-bound Blob custody и commit-before-Ack.
AI contract unit имеет concrete reply request/result, common context receipt,
deterministic request digest и provider-neutral bounded local-only generation
port. Provider result обязан вернуть typed completeness и bounded confidence;
engine не фабрикует эти значения. Всё перечисленное имеет Cargo/architecture
evidence. Live managed conformance запускает Communications с настоящими
Vault/Storage/Blob/NATS, передаёт bounded sender/subject/body только через
target-bound Blob receipt, не публикует private content в envelope, подавляет
duplicate result и fail-closed отклоняет stale и inactive source revision.
Gate `communications_ai_context_source_v1` реализован.
`makosh-ai-inference-core` также реализован как отдельная engine unit с
revision-fenced lifecycle, fixed prompt/policy receipt и sanitized terminal
results. `makosh-ai-inference-persistence` реализован отдельной owner-local
PostgreSQL unit: typed lifecycle, request/source receipts, provider-reported
effective settings revision, recoverable runs и terminal candidate сохраняются без
source message body или cross-owner SQL. `makosh-ai-inference-runtime`
реализует exact managed `request_rpc`, target-bound Blob custody/read,
provider-neutral outbound route и restart recovery; отдельный
`makosh-ai-inference-assembly` материализует только unsigned descriptor,
settings schema и Storage bundle inputs. Все пять AI engine build units
реализованы. Signed managed conformance запускает eventless AI Engine с
настоящими Vault/Storage/Blob, через Kernel module-request router маршрутизирует
typed provider request в отдельную Ollama integration, сохраняет terminal
`ProviderUnavailable`, после engine restart возвращает exact persisted
response без повторного provider request и отклоняет request-ID conflict. Это
доказывает admission, Blob custody, storage binding и отрицательный replay
contour. Дополнительный live managed conformance запускает тот же signed AI
Engine с настоящими Vault/Storage/Blob и отдельной Ollama integration против
реального provider process: target-bound source materialization, два
последовательных `request_rpc`, typed `Ready` result и exact positive replay
после остановки provider и restart engine подтверждены. Engine связывает
request и recovery с authenticated human owner; wrong-owner delivery
отклоняется до persistence, а recovery SQL выбирает только runs этого owner.
Gate `ai_inference_v1` реализован независимо от Communications, reply workflow
и Ollama implementation. Для Ollama реализованы все шесть отдельных staged
units —
`makosh-ollama-ai-api`,
`makosh-ollama-ai-core`, `makosh-ollama-ai-http`,
`makosh-ollama-ai-persistence`, `makosh-ollama-ai-runtime` и
`makosh-ollama-ai-assembly`: exact non-secret settings, fixed
loopback/model policy, request digest, structured result, owner-local
revision-fenced PostgreSQL lifecycle и terminal `uncertain` transition без
automatic retry. Persistence не хранит source content, prompt или HTTP request
body. HTTP unit реализует bounded `/api/tags` discovery и `/api/chat` dialect,
exact model binding, fixed JSON/non-streaming/non-thinking request и
закрытая typed JSON Schema для обязательных `subject/body/language`, а также
fail-closed response framing без redirects. Managed integration runtime
реализует exact provider request RPC, commit-before-HTTP lifecycle, model
digest confirmation после generation и crash recovery только в `uncertain`,
без повторной отправки. Assembly материализует только unsigned descriptor,
settings schema, owner-local Storage bundle и release fragment. Dev release
compiler включает exact runtime и Storage artifacts в подписанный distribution
manifest. Real managed conformance запускает этот signed eventless Integration
через Kernel с настоящими Vault/Storage и disposable PostgreSQL/PgBouncer,
фиксирует terminal `ProviderUnavailable`, после restart возвращает exact
persisted response без второй HTTP-попытки и отклоняет request-ID conflict.
Дополнительный live managed conformance использует отдельный официальный
Ollama process и реальную локальную модель: exact `/api/tags` discovery,
`/api/chat`, повторное подтверждение model digest и typed `Ready` result
проходят через signed Integration runtime. Provider dialect нормализует только
ASCII-регистр трёх закрытых language tokens; whitespace, aliases и free-form
значения отклоняются. Runtime связывает request tenancy с authenticated human
owner из managed launch и отклоняет wrong-owner delivery до persistence или
provider HTTP. Gate `ollama_ai_provider_v1` реализован независимо от
Communications, AI Engine и reply workflow. Для reply workflow реализованы все
пять отдельных units:
`makosh-communication-reply-suggestion-api` с concrete generated
Start/Get/realtime contract и
`makosh-communication-reply-suggestion-core` с revision/digest-fenced
state machine, а также `makosh-communication-reply-suggestion-persistence` с
owner-local idempotent run state, source-result inbox/hash fence, exact
source-prepare outbox, recoverable state и client-safe realtime replay.
`makosh-communication-reply-suggestion-runtime` реализует managed Workflow
admission, event-only source consumption, отдельную target-bound Blob
materialization для AI, exact inference `request_rpc`, terminal cleanup до Ack
и client-safe invalidation через общий replayable SSE. Отдельная
`makosh-communication-reply-suggestion-assembly` materializes только unsigned
descriptor, settings schema, owner-local Storage bundle и release fragment;
dev release compiler включает exact runtime и Storage artifacts в подписанный
distribution manifest. Persistence не хранит source body, prompt или provider
metadata. ADR-0357 реализовал Communications source revision 2 с bounded typed
sender/subject/body content без provider facade. Signed Kernel admission и
live negative orchestration теперь реализованы отдельным full-ensemble
conformance: реальные Vault/Storage/Blob/NATS, Communications domain, Reply
Suggestion workflow, AI engine и Ollama integration запускаются как отдельные
signed managed процессы; Start/Get проходят через Gateway, source передаётся
только durable events и target-bound Blob, а inference проходит через два
последовательных Kernel `request_rpc`. Observable loopback listener доказывает
реальную Ollama HTTP-попытку. Terminal `InferenceRejected` приходит через
replayable SSE без sender/subject/body или source identity; exact operation
replay после workflow restart не повторяет HTTP, conflicting operation
отклоняется, а stale source завершается `SourceRejected` до provider boundary.
Заодно Storage authority workflow исправлена: human logical owner больше не
подменяет owner namespace отдельной Storage unit, а временный
`RUNTIME_UNAVAILABLE` Gateway отображает как 503, не как internal 500.
Дополнительный positive full-ensemble conformance запускает тот же signed
контур против отдельного настоящего Ollama process и локальной модели:
authenticated Gateway Start/Get доходит через Communications event source,
target-bound Blob, AI Engine и Ollama Integration до typed `Ready` candidate.
Terminal state приходит через replayable SSE без private source content и
source identity. После остановки provider и restart только workflow exact
query/event/operation replay возвращается из owner-local persistence без нового
provider request. Wrong-owner client delivery отклоняется runtime до
persistence и provider boundary. Owner-authorized revoke повышает grant epoch,
переводит только workflow Storage binding в `Revoking`, останавливает только
Reply Suggestion process, сохраняет Communications, AI Engine и Ollama
Integration активными и удаляет Gateway capability route с fail-closed `404`.
Gate `communication_reply_suggestion_v1` реализован независимо от
Communications domain, AI Engine и Ollama Integration.

Уточняет:

- [ADR-0200](ADR-0200-clean-room-module-model-and-runtime-isolation.md);
- [ADR-0204](ADR-0204-bundled-integration-plugins-and-provider-neutral-context-boundary.md);
- [ADR-0205](ADR-0205-core-gateway-and-client-transport.md);
- [ADR-0212](ADR-0212-crate-topology-and-compile-isolation.md);
- [ADR-0213](ADR-0213-code-ownership-and-module-autonomy.md);
- [ADR-0215](ADR-0215-open-module-registration-and-capability-grants.md);
- [ADR-0220](ADR-0220-canonical-durable-envelope-and-contract-evolution.md);
- [ADR-0222](ADR-0222-kernel-settings-registry-and-supervised-reconfiguration.md);
- [ADR-0223](ADR-0223-encrypted-sqlite-vault-and-scoped-credential-leases.md);
- [ADR-0226](ADR-0226-ai-context-acquisition-through-use-case-workflows.md);
- [ADR-0231](ADR-0231-kernel-blob-service-and-owner-scoped-custody.md);
- [ADR-0253](ADR-0253-communications-legacy-surface-disposition-and-clean-room-completion.md);
- [ADR-0282](ADR-0282-full-communications-and-settings-capability-reconstruction.md);
- [ADR-0315](ADR-0315-communications-message-body-content-read.md);
- [ADR-0339](ADR-0339-capability-routed-module-request-rpc.md);
- [ADR-0350](ADR-0350-explicit-human-owner-context-for-managed-domain-and-integration-runtimes.md);
- [ADR-0354](ADR-0354-integration-implemented-request-rpc-extension-ports.md);
- [ADR-0355](ADR-0355-capability-scoped-integration-event-hub-launch-configuration.md).

## Контекст

На момент принятия решения clean-room inventory требовал
`communication_reply_suggestion_v1`, но active backend ещё не имел AI public
contract, inference owner, provider adapter или разрешённого module-to-module
body handoff. Текущее состояние реализации описано выше.

Существующий `communications_content_read_v1` нельзя переиспользовать как
workflow port. Он выдаёт one-use capability только authenticated client session,
имеет `client_blob` transport и прямо исключает AI context. Передача этого
ticket другому runtime нарушила бы authority binding и превратила бы Gateway в
private-content facade.

Historical AI Reply подтверждает только product semantics:

- один canonical message является required source;
- caller явно выбирает tone и language;
- результат содержит reply subject и body;
- candidate сначала показывается для review и только затем может быть передан
  compose workflow;
- variants являются отдельным bounded fan-out, а не скрытым default behavior.

Legacy REST routes, shared in-process `AiRuntimePort`, prompt strings,
Communications-owned model selection и fallback `Ok(None)` не являются
clean-room контрактами.

## Решение

### Четыре owners/gates, а не один facade

Вертикальный срез состоит из четырёх независимо принимаемых gates:

```text
Communications domain
  communications_ai_context_source_v1
        |
        | target-bound source receipt
        v
communication_reply_suggestion workflow
        |
        | exact CommunicationReplySuggestionInferenceRequestV1
        v
AI inference engine
  ai_inference_v1
        |
        | typed local provider generation request
        v
Ollama integration
  ollama_ai_provider_v1
```

Kernel и Gateway согласуют descriptor, grants, routes, runtime generations,
storage/settings bindings и hard bounds. Они не импортируют ни один из этих
owner packages, не читают message body, не строят prompt и не выбирают модель.

Domain, workflow, engine и integration являются разными owners и разными
единицами сборки. Совпадение процесса разработки или малый размер кода не
разрешает объединить их.

### Communications-owned source handoff

`communications_ai_context_source_v1` добавляет public contract unit
`makosh-communications-ai-source-api`. Communications runtime реализует exact
durable prepare command/result:

```text
communications / ai_reply_source_prepare / v1
communications / ai_reply_source_prepared / v1
```

Prepare command содержит только:

- 16-byte workflow run ID;
- 16-byte canonical message ID;
- expected canonical message revision;
- target owner/runtime/capability binding для
  `communication_reply_suggestion`;
- correlation/causation metadata.

Он не содержит provider, account, body, subject, participant address, Blob
locator или arbitrary purpose.

Communications:

1. проверяет inbox ID/hash до mutation;
2. авторизует logical human owner и exact current canonical revision;
3. требует non-deleted message и admitted UTF-8 body;
4. создаёт bounded target-bound Blob source copy;
5. сохраняет preparation result и exact outbox bytes атомарно;
6. публикует только typed metadata, declared size/digest и opaque
   evidence-bound custody proof.

Client content ticket из ADR-0315 не используется. Provider fallback,
cross-owner SQL, direct socket и body bytes в NATS запрещены.

### AI public contracts

Public unit `makosh-ai-contracts` принадлежит engine owner `ai` и содержит:

- общий `AiContextReceiptV1`;
- exact `CommunicationReplySuggestionInferenceRequestV1`;
- exact `CommunicationReplySuggestionInferenceResultV1`;
- provider-neutral generation port для approved AI provider integrations.

Это не generic context API. Reply request имеет concrete fields:

- receipt;
- target-bound source reference;
- normalized tone enum;
- normalized language enum;
- reply subject policy;
- target-bound typed private source content with bounded provider-neutral
  sender, subject and body needed by reply semantics;
- maximum output bytes/tokens;
- local-only egress policy revision.

Запрещены stringly typed model/provider identity, arbitrary prompt, maps,
Protobuf `Any`, opaque business payload и repeated heterogeneous fragments.
Schema digest, contract revision и deterministic request digest входят в
receipt.

Inference result содержит только:

- workflow run ID;
- exact request/context digest;
- bounded UTF-8 subject/body candidate;
- resolved language and tone;
- model/prompt/policy receipt without credentials or private model response;
- completeness, confidence and sanitized terminal status.

Candidate не является Communication, draft или provider command.

### AI inference engine

`ai_inference_v1` принадлежит engine owner `ai`:

```text
makosh-ai-contracts
makosh-ai-inference-core
makosh-ai-inference-persistence
makosh-ai-inference-runtime
makosh-ai-inference-assembly
```

Core валидирует concrete request, budgets, policy and result. Persistence
хранит run lifecycle, request/source digests, selected settings revisions,
sanitized failures and typed result, но не долговечную копию message body или
generic context cache. Runtime принимает only exact AI requests, читает
target-bound source bytes и вызывает единственный descriptor-approved Ollama
provider contract через capability router. Multi-provider routing и AI-owned
runtime settings не входят в V1 и требуют отдельного gate.
Core применяет deterministic UTF-8-safe context framing и bounded 2000-byte
body excerpt, сохраняя reference-поведение без unsafe Unicode slicing; поэтому
private provider request остаётся внутри конституционного 64 KiB/30-second
`request_rpc` bound и не расширяет platform transport скрытым исключением.

AI engine:

- не импортирует Communications или workflow implementation;
- не вызывает Communications query API;
- не получает cross-owner SQL;
- не принимает caller-selected provider/model;
- не имеет hidden generic module-settings apply;
- не выдаёт credentials provider runtime;
- не записывает business truth;
- не выполняет automatic remote fallback.

Первая production revision имеет только `local_only` egress. Remote provider
egress требует отдельного ADR с explicit consent, redaction and credential
lease evidence.

### Ollama integration

`ollama_ai_provider_v1` принадлежит integration owner `ollama`:

```text
makosh-ollama-ai-api
makosh-ollama-ai-core
makosh-ollama-ai-http
makosh-ollama-ai-persistence
makosh-ollama-ai-runtime
makosh-ollama-ai-assembly
```

Integration владеет Ollama HTTP dialect, endpoint validation, model discovery,
timeouts, response framing and provider errors. Она реализует только approved
AI provider contract и не импортирует Communications, reply workflow or AI
engine implementation.

V1 допускает только loopback Ollama endpoint. Redirects, non-loopback target,
caller URL, automatic model download и implicit model substitution запрещены.
Endpoint и model selection приходят только из Ollama-owned effective settings,
согласованных Kernel Settings Registry через существующий integration settings
apply. Availability не означает permission to download.

Persistence хранит только request ID/digest, settings revision, selected model
digest и bounded terminal provider result. Она не хранит private prompt/input,
HTTP body, credentials или model response envelope. Exact replay того же
request ID/digest возвращает сохранённый terminal result; другой digest для
того же ID отклоняется. После неоднозначного HTTP outcome run остаётся typed
`uncertain` и не отправляется в Ollama повторно автоматически, потому что
Ollama `/api/chat` не предоставляет доказанного idempotency key.

Private content передаётся integration runtime только через bounded typed local
`request_rpc`; оно не входит в NATS, logs, traces, health или settings.

### Reply-suggestion workflow

Owner `communication_reply_suggestion` имеет отдельные units:

```text
makosh-communication-reply-suggestion-api
makosh-communication-reply-suggestion-core
makosh-communication-reply-suggestion-persistence
makosh-communication-reply-suggestion-runtime
makosh-communication-reply-suggestion-assembly
```

Client contract предоставляет:

```text
StartReplySuggestion(message_id, expected_revision, tone, language)
  -> accepted run_id

GetReplySuggestion(run_id)
  -> pending | ready(candidate) | rejected
```

V1 поддерживает exact tone enum `professional | friendly | concise | formal`
и language enum `source | english | russian | spanish`. Free-form prompt,
arbitrary language/model/provider, variants matrix и automatic Compose mutation
не входят в gate.

Workflow:

1. атомарно сохраняет idempotent run;
2. публикует Communications source prepare command;
3. принимает source result через inbox/hash fence;
4. собирает concrete AI request и `AiContextReceiptV1`;
5. вызывает exact AI inference request through `request_rpc`;
6. проверяет returned request/context digest;
7. сохраняет terminal candidate и client-safe realtime invalidation.

Frontend review использует только generated workflow client. Apply-to-compose
позже передаёт approved candidate отдельному compose/delivery workflow; он не
является частью inference и не мутирует Communications.

### Durability, privacy and fencing

- Start acceptance не означает source preparation или inference completion.
- Operation/run ID обеспечивает idempotency; payload hash mismatch rejected.
- Every owner хранит свой inbox/outbox/state only in its Storage namespace.
- Runtime generation, grant epoch, settings revision and Blob custody proof
  проверяются на каждом authority boundary.
- Timeout after ambiguous provider call не повторяется автоматически без same
  run/provider idempotency evidence; state становится typed `uncertain`.
- Restart resumes durable accepted work without creating a second candidate.
- SSE содержит только run ID, state, revision and occurred time.
- Message body, candidate body, prompt, provider response, Blob proof, model
  endpoint and errors do not enter SSE/logs/health.
- Wrong human owner, stale message revision, invalid UTF-8, expired custody,
  revoke, oversize input/output and digest mismatch fail closed.

## Phase gates

### `communications_ai_context_source_v1`

Реализован: public source contract, Communications-owned inbox/outbox,
target-bound Blob custody, stale/edit/inactive negative matrix and live
event-only preparation evidence проверяются managed conformance.

### `ai_inference_v1`

Реализован: five AI engine units, common receipt and exact reply request/result,
owner-local run state, settings/fencing, target-bound Blob materialization,
provider-neutral `request_rpc`, restart/idempotency/privacy negatives,
authenticated human-owner fence, owner-scoped recovery и live managed
successful inference evidence проверяются conformance.

### `ollama_ai_provider_v1`

Реализован: separate API/core/http/persistence/runtime/assembly units,
loopback endpoint guard, exact settings, request digest/idempotency/uncertain
fencing, model/timeout/error conformance, private-content non-disclosure,
authenticated human-owner fencing и live request к отдельному настоящему
Ollama process проверяются managed conformance. Mock or canned response не
является production evidence.

### `communication_reply_suggestion_v1`

Реализован: five workflow units, generated Start/Get/realtime contracts,
durable event-only source orchestration, target-bound Blob custody, separate AI
Engine request RPC, exact typed candidate, authenticated Gateway/SSE flow,
restart/replay, wrong-owner, revoke, stale-source, provider-unavailable и
privacy conformance. Communications, workflow, AI Engine и Ollama Integration
остаются отдельными build/runtime owners.

Наличие ADR, prompt unit test, skeleton или frontend card не открывает ни один
gate.

## Отклонённые варианты

### AI Reply method в Communications

Смешивает canonical evidence domain, workflow orchestration, model policy and
provider execution.

### Gateway fetches body and calls AI

Превращает transport boundary в private-content and AI facade.

### Workflow reuses client content ticket

Нарушает session/recipient binding ADR-0315 и скрывает новый cross-owner grant.

### AI engine queries Communications

Делает inference owner cross-domain orchestrator, запрещённый ADR-0226.

### Ollama adapter inside AI engine

Смешивает engine and integration build units and provider lifecycle.

### Return empty candidate when runtime is unavailable

Маскирует missing authority/runtime как successful AI result. V1 возвращает
typed unavailable/rejected/uncertain state.
