# ADR-0354: Integration-implemented request RPC extension ports

Статус: Принято

Дата: 2026-07-31

Состояние реализации: реализовано. Descriptor admission разрешает только
`Integration` module реализовать exact foreign-owned `request_rpc` contract.
Domain, workflow, engine, service и platform modules не могут объявить такой
provider surface; same-owner правила для `query_rpc`, `client_rpc`,
`client_realtime` и business event authority не изменены. Kernel registration,
approval gating и Ollama managed negative conformance являются executable
evidence. Live успешный Ollama inference остаётся отдельным gate ADR-0353.

Уточняет:

- [ADR-0204](ADR-0204-bundled-integration-plugins-and-provider-neutral-context-boundary.md);
- [ADR-0215](ADR-0215-open-module-registration-and-capability-grants.md);
- [ADR-0221](ADR-0221-module-descriptor-and-capability-lifecycle-contract.md);
- [ADR-0339](ADR-0339-capability-routed-module-request-rpc.md);
- [ADR-0353](ADR-0353-communication-reply-suggestion-and-ai-inference-boundary.md).

## Контекст

ADR-0339 первоначально требовал, чтобы owner каждого provided `request_rpc`
contract совпадал с owner provider module. Это корректно для domain-owned
application commands, но не позволяет реализовать typed extension port:

```text
AI engine owns provider-neutral generation contract
        |
        | exact request_rpc dependency
        v
Ollama integration implements provider execution
```

Перенос contract owner к `ollama` заставил бы AI engine выбирать provider
schema и смешал бы engine port с integration ownership. Перенос Ollama runtime
к owner `ai` смешал бы integration, settings, storage и release authority с
engine. Ослабление owner check для всех module kinds позволило бы domain или
workflow объявить чужой application contract своим provider surface.

## Решение

### Узкое delegated implementation rule

Foreign-owned provided `request_rpc` допустим только когда:

1. descriptor имеет `ModuleKindV1::Integration`;
2. contract reference полностью exact:
   owner/name/major/revision/schema SHA-256;
3. provider capability явно присутствует в descriptor;
4. owner отдельно approve-ит exact capability;
5. caller имеет exact contract dependency и granted capability;
6. Kernel на каждом вызове проверяет current registration, runtime generation,
   grant epoch, единственного provider и logical human owner;
7. Kernel остаётся opaque relay и не декодирует payload;
8. integration не получает contract ownership и не создаёт business truth.

Contract owner сохраняет authority над schema и semantics. Integration
реализует только provider execution port. Совпадение Rust message types не
создаёт compile-time dependency на engine implementation или storage.

### Неизменившиеся same-owner surfaces

Foreign ownership остаётся запрещённым для:

- domain/workflow/engine/service `request_rpc` providers;
- всех `query_rpc` providers;
- `client_rpc`, `client_blob` и `client_realtime`;
- canonical events, observations и durable business commands;
- settings, storage namespace и credential ownership.

Domain не может использовать это правило для вызова другого domain. Cross-
domain business flow по-прежнему идёт event → workflow/consumer → target
command → target event.

### Первый consumer

`makosh-ollama-ai-runtime` остаётся integration owner `ollama` и реализует exact
AI provider generation port. AI inference engine объявляет exact dependency,
но не импортирует Ollama implementation, persistence или HTTP adapter. Ollama
integration не импортирует Communications domain и не записывает AI candidate
как business truth.

## Units и SRP

```text
AI public contract unit
  request/result schema and provider-neutral semantics

Ollama integration API/core/http/persistence/runtime/assembly
  one provider implementation and owner-local lifecycle

Kernel descriptor admission
  exact delegated implementation authorization only
```

Ни одна из этих responsibilities не объединяется ради удобства routing.

## Phase gate

Решение считается реализованным только при наличии:

1. negative test: Domain не может предоставить foreign-owned request contract;
2. positive test: pending Integration route не виден до approval;
3. exact approved provider inventory после approval;
4. existing zero/ambiguous/stale/revoke/fence request routing negatives;
5. managed signed integration admission через real Vault/Storage;
6. architecture, Cargo, Clippy и test gates.

Принятый ADR без executable evidence gate не открывает.

## Отклонённые варианты

### Сделать Ollama module owner `ai`

Смешивает integration storage/settings/release authority с AI engine.

### Сделать AI engine зависимым от Ollama-owned contract

Встраивает provider identity в engine port и закрывает заменяемость на уровне
capability routing.

### Разрешить foreign contracts всем module kinds

Позволяет domain или workflow объявить чужую application authority и разрушает
owner boundary.
