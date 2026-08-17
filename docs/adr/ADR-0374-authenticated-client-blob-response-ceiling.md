# ADR-0374: Authenticated client Blob response ceiling

Статус: Принято

Дата: 2026-08-01

Состояние реализации: implemented in protocol validation, Kernel Control Store
schema v49 and Gateway route admission. Managed Attachment Preview proof remains
part of the separate `attachment_preview_v1` gate. ADR-0413 сохраняет этот
32 MiB ceiling как предел inline buffered response, но вводит отдельный 4 GiB
object ceiling для session-bound bounded range delivery и schema v50.

Зависит от:

- [ADR-0205](ADR-0205-core-gateway-and-client-transport.md);
- [ADR-0230](ADR-0230-blob-platform-opaque-references-and-owner-local-metadata.md);
- [ADR-0318](ADR-0318-communications-evidence-export-workflow.md);
- [ADR-0373](ADR-0373-bounded-attachment-preview-workflow.md).
- [ADR-0413](ADR-0413-chunked-blob-storage-and-session-bound-client-range-delivery.md).

## Контекст

ADR-0318 ввёл platform-wide ceiling `24 MiB` для одного authenticated
`client_blob` ответа. Это было достаточно для Communications Export и не было
выведено из transport или Blob protocol limit. ADR-0373 отдельно допускает
bounded MP4 presentation copy до `32 MiB`. Его exact descriptor не может быть
admitted при старом platform ceiling, хотя Blob data-plane уже имеет более
строгий отдельный общий предел `64 MiB`.

Уменьшение MP4 preview до `24 MiB` изменило бы принятое product contract без
security причины. Выдача bytes через JSON, base64 или provider URL запрещена и
не является обходным вариантом.

## Решение

Platform-wide hard ceiling одного inline authenticated `client_blob` ответа
повышается с `24 MiB` до `32 MiB`. ADR-0413 позднее отделяет этот memory bound
от descriptor object ceiling для range transport.

Изменение применяется атомарно в четырёх authority:

1. `ModuleDescriptorV1` validation;
2. private Kernel Control Store schema и route admission;
3. Core Gateway runtime route admission;
4. architecture regression guard.

Kernel Control Store получает forward-only schema migration v48 -> v49,
которая сохраняет existing route rows и меняет только CHECK bound. Existing
Communications Export route остаётся ограничен собственным `24 MiB` contract;
новый ceiling не расширяет его descriptor автоматически.

Каждый module route по-прежнему обязан объявить exact меньший либо равный
`max_response_bytes`. Gateway требует authenticated owner/device session,
exact registered route and contract, runtime/grant fences и возвращает private
bytes с `Cache-Control: no-store`. Module должен выдать одноразовую bounded
authorization; internal Blob reference, receipt и grant клиенту не выдаются.

Не меняются:

- inline response buffering limit `32 MiB`;
- module Blob quota и custody scope;
- buffering semantics существующих inline routes;
- route-specific bounds существующих modules;
- запрет generic Blob route и provider URLs.

## Последствия

Attachment Preview может объявить exact `32 MiB` MP4 `client_blob` route без
расширения Export или другого owner. Control Store upgrade сохраняет
registrations и не требует reset. Память Gateway остаётся bounded hard ceiling;
streaming transport является отдельным будущим решением и не подразумевается
этим ADR.

## Отклонённые варианты

### Понизить MP4 preview до 24 MiB

Отклонено: это тихо расходится с ADR-0373 и не следует из data-plane limit.

### Вернуть bytes через JSON или base64

Отклонено: private content попал бы в business RPC, heap и logging surfaces.

### Разрешить route без общего hard ceiling

Отклонено: owner-controlled descriptor не должен определять platform memory
bound без Kernel/Gateway maximum.
