# ADR-0375: Static Preview renderer admission and failure semantics

Статус: Принято

Дата: 2026-08-02

Состояние реализации: реализовано. Preview V1 renderer adapters являются
отдельными compile-isolated build units, статически связанными с exact signed
managed runtime. Runtime identity включает все adapter packages и pinned DOCX
font digest. Недостижимые внутренние renderer outcomes `Unavailable` и
`TimedOut` удалены; отсутствие или подмена adapter отсекаются до `ready` через
Cargo/release inventory, exact executable digest и managed admission. Отдельный
renderer process, environment test hook или fake outage не вводятся.

Зависит от:

- [ADR-0212](ADR-0212-crate-topology-and-compile-isolation.md);
- [ADR-0219](ADR-0219-managed-module-distribution-integrity-and-explicit-updates.md);
- [ADR-0372](ADR-0372-kernel-staged-runtime-resources-for-managed-workflows.md);
- [ADR-0373](ADR-0373-bounded-attachment-preview-workflow.md).

## Контекст

ADR-0373 первоначально требовал negative evidence для `unavailable renderer`.
Реализация V1 при этом сознательно выбрала pure-Rust adapters без shell,
network, native dynamic libraries и runtime data files:

```text
makosh-attachment-preview-text
makosh-attachment-preview-image
makosh-attachment-preview-pdf
makosh-attachment-preview-docx
makosh-attachment-preview-media
```

Все пять packages являются отдельными единицами сборки по функциональной
ответственности, но не отдельными OS-процессами. Они статически связаны с
`makosh-attachment-preview-runtime`. DOCX font включён compile-time bytes с
pinned digest. После успешной проверки exact runtime executable отдельного
renderer lifecycle, socket, staged executable или mutable model resource нет.

Создать managed «renderer outage» можно было бы только через test-only
environment switch, подменный adapter или искусственный optional registry.
Такая проверка не соответствовала бы production topology и стала бы fake
evidence. Превращать adapters в процессы только ради outage-теста также
нарушило бы минимальную runtime topology без продуктовой причины.

## Решение

Для Preview V1 renderer availability является admission invariant, а не
runtime health state:

```text
exact Cargo package inventory
∩ compile isolation
∩ signed release executable digest
∩ renderer identity over adapter versions and fixed resources
∩ managed launch admission
= complete renderer set before ready
```

Missing adapter делает runtime несобираемым. Подменённый или неполный runtime
не проходит signed manifest/executable verification. Stale renderer identity
делает ранее созданный artifact нечитаемым. Эти три независимые границы
заменяют несуществующий process-outage scenario.

Внутренний `AttachmentPreviewRendererErrorV1` содержит только outcomes, которые
реально может вернуть byte-only synchronous V1 adapter:

- `Empty`;
- `SourceTooLarge`;
- `Unsupported`;
- `InvalidContent`;
- `OutputTooLarge`;
- `Failed`.

`Unavailable` и `TimedOut` удаляются как недостижимые состояния. Public client
enum не расширяет authority и может сохранять bounded
`renderer_unavailable` для будущего versioned adapter gate, но V1 runtime его
не синтезирует и не использует как fallback.

Будущий renderer с native executable, model/data resource, subprocess или
отдельным managed process требует нового ADR и exact gate по ADR-0372. Он не
может тихо использовать Preview V1 descriptor, settings, grants или release
inventory.

## Failure semantics

- malformed/active/polyglot input возвращает exact typed content outcome;
- bounded adapter failure возвращает `renderer_failed` без partial artifact;
- missing/tampered adapter binary fail-close-ится до runtime `ready`;
- stale renderer identity блокирует ticket/client Blob read;
- runtime crash остаётся Kernel-managed runtime failure, а не выдуманным
  renderer outage;
- никакой fallback на source bytes, browser rendering или другой adapter не
  допускается.

## Build units и SRP

Adapter packages остаются отдельными build units. Dispatcher только выбирает
adapter по verified magic. Runtime владеет orchestration/job lifecycle, но не
реализует PDF, DOCX, image или media parsing. Assembly владеет unsigned release
composition, но не renderer execution. Kernel проверяет executable bytes, не
интерпретируя renderer semantics.

## Phase gate

Решение доказано только при одновременном выполнении:

1. exact five-adapter compile-isolated inventory;
2. runtime dependency allowlist без optional/dynamic fallback;
3. renderer identity включает каждый adapter package и fixed resource digest;
4. signed managed runtime launch проходит exact executable verification;
5. stale renderer identity fail-close-ит artifact read;
6. internal contract не заявляет недостижимые unavailable/timeout outcomes;
7. architecture, SRP, Cargo and managed Preview conformance проходят.

## Отклонённые варианты

### Test-only environment outage

Отклонено: доказывает специальный тестовый код, которого нет в production
topology.

### Optional adapter registry

Отклонено: создаёт частично готовый runtime и скрытый fallback вместо exact
release inventory.

### Отдельный renderer process без иной причины

Отклонено: добавляет supervisor/grant/IPC lifecycle только ради искусственного
failure mode.

### Browser rendering исходного документа

Отклонено: переносит untrusted active content и private bytes через client
boundary и обходит derived Blob custody.
