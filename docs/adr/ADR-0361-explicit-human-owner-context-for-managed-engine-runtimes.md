# ADR-0361: Explicit human owner context for managed Engine runtimes

Статус: Принято

Дата: 2026-07-31

Состояние реализации: реализовано для managed Engine launch protocol, Kernel
staging и Attachment Archive Inspection tenancy. Live managed conformance
подтверждает distinct module/human owner authority через Gateway, Storage,
Event Hub и replayable SSE.

Уточняет:

- [ADR-0215](ADR-0215-open-module-registration-and-capability-grants.md);
- [ADR-0224](ADR-0224-storage-control-plane-owner-scoped-postgresql-and-migration-lifecycle.md);
- [ADR-0350](ADR-0350-explicit-human-owner-context-for-managed-domain-and-integration-runtimes.md);
- [ADR-0358](ADR-0358-capability-scoped-engine-event-hub-launch-configuration.md);
- [ADR-0359](ADR-0359-bounded-attachment-archive-inspection-engine.md).

## Контекст

`ManagedEngineRuntimeConfigurationV1.logical_owner_id` обозначает владельца
Engine build unit. Kernel использует его для registration, grants, Storage
binding и Event Hub authority. Это не identity человека, которому принадлежат
private input, owner-local state и client realtime.

Пока Engine не имел client-facing owner-local состояния, отдельный human owner
ему не требовался. Attachment Archive Inspection принимает Gateway Start/Get,
durably объединяет private attachment evidence и публикует replayable client
status. Использование `attachment_archive_inspection` как tenancy key смешало
бы module authority с пользовательской authority.

## Решение

Managed Engine launch contract получает отдельное обязательное поле
`logical_human_owner_id`.

```text
logical_owner_id
    = Engine build-unit owner
    = registration / grants / Storage / Event Hub authority

logical_human_owner_id
    = authenticated human owner
    = business tenancy / client request / client realtime authority
```

Kernel формирует human owner только из авторизованной owner session,
использованной для exact Engine launch request. Settings, event payload,
provider data и сам runtime не могут выбирать или подменять это значение.

Structural protocol validation требует bounded canonical identity. Module
owner не используется как fallback, а human owner не заменяет owner в Storage
binding и не расширяет GrantSet.

## Runtime boundary

- generic managed Engine protocol переносит обе identity явно;
- Kernel staging связывает human owner с текущей авторизованной session;
- Engine использует module owner для managed control, Event Hub permit и
  owner-local build-unit Storage binding;
- client-facing Engine использует human owner для tenant keys, Gateway
  authorization и realtime delivery;
- typed durable business contract продолжает владеть своей owner semantics;
  Kernel, Gateway и Event Hub не декодируют payload;
- Engine, который не имеет owner-local business state, всё равно получает
  authenticated human context, но не обязан создавать фиктивную проекцию или
  client surface.

## Units и SRP

Решение не создаёт общий owner facade:

- `makosh-runtime-protocol` владеет wire field и structural validation;
- Kernel owner-control dispatch владеет authenticated staging;
- каждый Engine runtime сам валидирует и применяет human owner только в своих
  owner-local responsibilities;
- persistence остаётся внутри соответствующего Engine owner package.

Engine остаётся отдельной единицей сборки и runtime failure boundary. Domain,
integration, workflow и Engine не объединяются.

## Failure semantics

- missing или malformed human owner отклоняет managed launch;
- stale owner session не может запустить Engine;
- module owner не принимается как неявный human owner;
- client request с другим authenticated owner не читает и не изменяет tenancy;
- restart сохраняет durable owner-local state, но получает новую runtime
  generation и заново подтверждённый owner context.

## Phase gate

Решение считается реализованным только после:

1. protocol positive/negative tests;
2. Kernel staging из authenticated owner session;
3. сохранения module owner для Storage/Event Hub authority;
4. Archive Inspection Start/Get/SSE под distinct human owner;
5. restart/replay и cross-owner privacy-negative conformance;
6. architecture, Cargo, Clippy и test gates.

## Отклонённые варианты

### Использовать Engine owner как human owner

Отклонено: смешивает build-unit authority и private tenancy.

### Передать human owner через settings

Отклонено: settings принадлежат модулю и не являются authentication context.

### Вывести owner из первого event или client request

Отклонено: untrusted data не может создавать runtime authority.
