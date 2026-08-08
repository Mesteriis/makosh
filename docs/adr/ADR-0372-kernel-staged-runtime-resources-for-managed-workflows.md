# ADR-0372: Kernel-staged runtime resources for managed workflows

Статус: Принято

Дата: 2026-08-01

Состояние реализации: реализовано для exact Attachment Text Extraction contour.
Owner-neutral protocol, exact use/kind validation, Kernel selector/staging, OCR
runtime binding и unsigned release fragment реализованы. Pinned macOS arm64
runner build, license/model hash audit, system-fallback negative и изолированная
двойная reproducibility проверка реализованы в отдельной native build unit.
Development signed release компилирует exact runtime/model artifacts, а
clean-room development assembly admit-ит и запускает Attachment Text Extraction
как отдельный managed workflow. Managed conformance запускает signed workflow
через Kernel с private staging exact runner/`eng`/`rus` resources и доказывает
реальный `eng+rus` job через Event/Blob flow, restart, outage, stale identity и
privacy-negative contours. Gate `attachment_text_extraction_v1` реализован;
другие workflow/engine runtime resources требуют собственных exact phase gates.

Зависит от:

- [ADR-0200](ADR-0200-clean-room-module-model-and-runtime-isolation.md);
- [ADR-0212](ADR-0212-crate-topology-and-compile-isolation.md);
- [ADR-0213](ADR-0213-code-ownership-and-module-autonomy.md);
- [ADR-0215](ADR-0215-open-module-registration-and-capability-grants.md);
- [ADR-0219](ADR-0219-managed-module-distribution-integrity-and-explicit-updates.md);
- [ADR-0221](ADR-0221-module-descriptor-and-capability-lifecycle-contract.md);
- [ADR-0267](ADR-0267-kernel-staged-runtime-artifacts-and-integration-state-roots.md);
- [ADR-0371](ADR-0371-bounded-attachment-text-extraction-workflow.md).

## Контекст

ADR-0371 требует реальный bounded OCR для `eng+rus`. Уже реализованный OCR
adapter принимает только абсолютный executable path, exact executable digest,
exact digests двух language models и private work directory. Runtime честно
fail-close-ится без такой конфигурации.

Передавать эти значения через Settings, environment, argv, системный `PATH` или
искать установленный Tesseract по convention нельзя:

- Settings принадлежат semantics модуля, а не release artifact admission;
- путь не доказывает exact bytes и current distribution binding;
- системный executable и соседние model files обходят signed release manifest;
- Kernel не должен знать OCR, Tesseract или language semantics;
- workflow не должен становиться integration только потому, что использует
  native executable;
- domain не должен получать parser/runtime dependency.

ADR-0267 уже определяет правильную authority ceremony, но ограничивает её
`ManagedIntegrationRuntimeConfigurationV1` и
`native_dynamic_library`. Это мешает provider-neutral workflow и engine
получать exact executable или immutable model data, хотя их descriptor и
release manifest уже являются общими Kernel contracts.

## Решение

Runtime artifact admission становится owner-neutral Kernel capability для
managed `integration`, `workflow` и `engine`. `domain` в V1 не получает
external runtime resources: business domain обязан оставаться независимым от
parser/provider/native execution.

Kernel не интерпретирует resource semantics. Он пересекает только:

```text
verified ModuleDescriptorV1 request
∩ effective owner-approved capability grant
∩ signed DistributionManifestV1 artifact bound to exact module_id
∩ current registration/runtime/grant generation
```

Результат передаётся через private inherited managed-runtime bootstrap channel.
Gateway, Event Hub, Settings Registry, client API, health и telemetry этот
binding не видят.

### Owner-neutral protocol unit

`ManagedRuntimeArtifactBindingV1` переносится из integration-specific proto в
отдельный runtime-protocol source unit. Это не compatibility facade: protobuf
type остаётся в том же package `makosh.runtime.v1`, а integration, workflow и
engine configurations импортируют один canonical definition.

`RuntimeArtifactUseV1` получает exact варианты:

```text
native_dynamic_library
native_executable
read_only_data
```

`RuntimeArtifactRequestV1` по-прежнему содержит только stable `artifact_id` и
exact `use`. Generic filesystem path, loader command, arguments, environment,
URL, provider identity или arbitrary metadata запрещены.

`ManagedWorkflowRuntimeConfigurationV1` и
`ManagedEngineRuntimeConfigurationV1` получают bounded ordered
`runtime_artifacts`. Integration продолжает использовать тот же canonical
field type. Empty list допустим только если ни одна effective capability не
запрашивает artifact.

### Signed distribution kinds

`DistributionArtifactKindV1` сохраняет существующий exact kind для dynamic
library и добавляет два отдельных bound-module kinds:

```text
module_runtime_native_executable
module_runtime_read_only_data
```

Каждый такой artifact имеет:

- globally unique stable `artifact_id` в distribution;
- exact `bound_module_id`;
- release-relative path;
- non-zero bounded size и SHA-256;
- exact target triple через parent manifest;
- required classification.

Executable/data artifacts не имеют descriptor или settings schema. Один
artifact ID не может быть привязан к нескольким modules. Use и distribution
kind должны совпадать exact; автоматический coercion запрещён.

### Kernel selection and staging

Один owner-neutral selector заменяет integration-only artifact selection. Он
принимает verified descriptor, exact granted capability IDs, signed manifest и
expected module kind. Selector:

1. отклоняет request для `domain`;
2. отклоняет неизвестный grant или request вне effective capability;
3. требует exact use/kind pair и exact `bound_module_id`;
4. сортирует и дедуплицирует только полностью одинаковые requests;
5. отклоняет один artifact ID с разными uses;
6. возвращает immutable requirements без filesystem path.

Перед initial launch, restart и successor generation Kernel повторно проверяет
installed signed distribution, создаёт private launch-scoped directory,
копирует exact bytes через create-new/no-symlink ceremony, сверяет inode,
size/digest до и после copy и создаёт one-shot binding:

```text
artifact_id
use
staged_path
size_bytes
sha256
```

Binding существует только для конкретных registration ID, runtime instance,
runtime generation и grant epoch. Cleanup принадлежит supervisor launch и
выполняется после stop/crash/failed launch. Runtime не получает исходный bundle
path.

Kernel staging не делает файл executable по умолчанию: mode зависит от exact
use. `native_executable` получает read+execute, `read_only_data` — read only,
dynamic library — существующую read/loader policy. Runtime обязан повторно
проверить regular-file identity, size и digest до использования. Ошибки не
содержат private path.

## Exact OCR binding

Attachment Text Extraction workflow запрашивает три artifacts в отдельной
required capability `attachment_text_extraction.ocr_runtime.v1`:

```text
attachment_text_extraction.ocr.runner.v1  native_executable
attachment_text_extraction.ocr.eng.v1     read_only_data
attachment_text_extraction.ocr.rus.v1     read_only_data
```

Runner является Макошь release artifact, собранным из pinned Tesseract и
Leptonica sources отдельной release build unit. Он должен быть self-contained:
non-system dynamic load commands запрещены; exact source revisions, patches,
license bytes, compiler/container digest и reproducibility evidence входят в
release gate. Runtime не использует Homebrew/system Tesseract fallback.

`eng.traineddata` и `rus.traineddata` являются отдельными exact data artifacts.
Runtime сопоставляет их только по artifact ID, повторно проверяет digest и
materializes private create-new `tessdata/eng.traineddata` и
`tessdata/rus.traineddata` под launch-scoped work root. Filename не приходит из
descriptor, Settings, event или клиента.

OCR adapter получает только проверенную runtime-owned конфигурацию. Language
policy остаётся compile-time `eng+rus`; пользовательский выбор модели, remote
download, auto-update и model fallback в V1 запрещены.

Три artifacts принадлежат release composition workflow owner, но не входят в
его production Cargo package inventory. Existing eleven production packages из
ADR-0371 сохраняются. Native runner build/verification является отдельной
release build unit и не становится domain, integration или runtime package.

`backend/scripts/build-attachment-text-extraction-ocr-macos.sh` принимает
только новый absolute output directory и optional
`--verify-reproducibility`. Build unit pin-ит exact commits Tesseract,
Leptonica, zlib, libpng и tessdata, exact model/license bytes, CMake archive и
Apple toolchain bytes. Developer `PATH` отсекается до system directories.
Кроме shell `PATH`, CMake system-environment lookup отключён, а package-manager
prefixes `/opt/homebrew` и `/usr/local` явно исключены из поиска.
Runner собирается со static non-system libraries, а release отвергается при
non-system Mach-O load command, `LC_RPATH` или побайтовом различии двух
изолированных builds. Только двойная проверка выставляет
`release_eligible=true` в provenance; single development build не является
release evidence.

## Failure semantics

Launch fail-close-ится до `ready`, если:

- required artifact отсутствует или не granted;
- use/kind/module binding не совпадает;
- manifest, size, digest, file identity или permission ceremony нарушены;
- artifacts не упорядочены, дублируются неоднозначно или stale;
- runtime получил missing/extra binding;
- runner или model повторная проверка не прошла;
- private work root нельзя создать или безопасно очистить.

Parser error не раскрывает path, model name из filesystem, stderr content или
private attachment bytes. Bounded sanitized error остаётся
`parser_unavailable`, `parser_failed` или `parser_timed_out`.

Restart/NATS outage/replay не создаёт второй parser run после terminal
owner-local commit. Новый runtime generation обязан получить новые staged paths
и bindings, даже если release digest не изменился.

## Build units и SRP

Ответственности разделены функционально:

```text
runtime protocol
  request/binding wire types and validation

distribution compiler
  exact signed artifact inventory only

Kernel selector/stager
  descriptor/grant/manifest intersection and private bytes

OCR native release build unit
  pinned source build, licenses and reproducibility

OCR adapter package
  bounded process execution and output validation

workflow runtime
  binding-to-adapter composition and job lifecycle

workflow assembly
  unsigned runtime/storage/native artifact fragment only
```

Kernel не импортирует OCR package. OCR adapter не импортирует Kernel,
workflow persistence, Communications, Attachment Security или integration.
Assembly не запускает runner и не подписывает manifest. Domain packages не
зависят ни от одного OCR/native unit.

## Phase gate

Решение считается реализованным только после:

1. neutral proto extraction без второго binding type;
2. exact use/kind validation и domain-negative tests;
3. workflow/engine/integration configuration validation;
4. owner-neutral selector and Kernel staging for all three admitted kinds;
5. restart/successor cleanup and stale-generation negative evidence;
6. exact unsigned OCR artifact fragment and signed release compilation;
7. pinned reproducible runner and license/dependency audit;
8. real `eng+rus` OCR through managed workflow;
9. wrong digest/use/module, missing/extra model, symlink and mutation negatives;
10. privacy-negative logs/errors/health/telemetry;
11. architecture, SRP, Cargo, clippy and full pre-push gates.

До этого `attachment_text_extraction_v1` и это решение остаются не доказанными
runtime evidence.

## Отклонённые варианты

### Передать Tesseract path и model directory через Settings

Отклонено: Settings не являются release integrity authority и создали бы
machine-local hidden configuration.

### Использовать системный Tesseract или Homebrew fallback

Отклонено: executable, libraries и models меняются вне signed distribution и
не связаны с current grant/runtime generation.

### Объявить workflow integration

Отклонено: OCR не является provider operational owner. Module kind не должен
определяться способом поставки native dependency.

### Перенести OCR в Communications или Attachment Security

Отклонено: canonical evidence, safety verdict и derived content processing —
разные authority и причины изменения.

### Публиковать image/text bytes через event

Отклонено: durable spine несёт только bounded typed evidence/custody metadata;
private content остаётся в Blob data plane.
