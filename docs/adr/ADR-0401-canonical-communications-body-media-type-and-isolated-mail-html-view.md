# ADR-0401: Canonical Communications body media type and isolated Mail HTML view

- Статус: принято
- Дата: 2026-08-04
- Состояние реализации: в реализации
- Связанные решения: ADR-0204, ADR-0205, ADR-0240, ADR-0257, ADR-0315,
  ADR-0397

## Контекст

Reference Mail viewer различал `body_text` и `body_html`. Clean-room перенос
сохранил только UTF-8 bytes без media type и для HTML-only писем заранее
превращал markup в visible text. В результате клиент не мог безопасно отличить
HTML от plain text и показывал либо плоский текст, либо unavailable state.

Запуск внешнего Google Chrome не является и не должен являться частью просмотра
письма. `make dev` предоставляет application URL, не открывая браузер по
умолчанию; message rendering живёт только внутри first-party client.

## Решение

Canonical body contract получает bounded provider-neutral media type:

- `communication_observed` повышается до revision 3; прежняя revision 2 не
  переиспользуется с другим schema hash;
- integration передаёт только `text/plain` или `text/html` вместе с Blob receipt;
- Communications сохраняет media type как часть evidence и возвращает его в
  short-lived content ticket, не раскрывая provider locator или Blob reference;
- body bytes остаются в Blob Platform и читаются через прежний one-use ticket;
- Mail выбирает HTML leaf для reference-rich view, plain text остаётся fallback;
- Mail body-observation identity повышается до `v6` и включает media type,
  digest содержимого и SHA-256 source-custody proof. Kernel продолжает
  отвергать proof после revoke или смены grant epoch; provider повторно
  materialize-ит body под текущим grant и публикует отдельную immutable
  canonical revision. Старый event не переписывается и Communications не
  вызывает Mail для repair;
- frontend декодирует UTF-8, пропускает HTML через email-specific sanitizer,
  блокирует remote images и рендерит результат в sandboxed iframe;
- scripts, forms, event handlers, embedded frames, unsafe URLs и remote loads
  запрещены; отсутствие готового canonical body показывает skeleton во время
  чтения и typed unavailable state только после terminal failure.

Communications не интерпретирует markup и не зависит от Mail. Media type —
provider-neutral descriptor canonical content, а sanitization является client
presentation boundary.

## Единицы сборки

- `hermes-mail-core`: bounded MIME leaf decoding;
- `hermes-mail-runtime`: provider ingestion и source Blob admission;
- `hermes-communications-ingress`: event contract;
- `hermes-communications-domain` / `persistence`: canonical descriptor custody;
- `hermes-communications-content-api` / runtime: owner content ticket;
- `frontend/src/workflows/mail-message-content`: app composition;
- `frontend/src/shared/sanitize` и Mail presentation: isolated rendering.

## Проверка

1. MIME tests доказывают HTML/plain selection и bounded UTF-8 decoding.
2. Ingress/domain/storage tests отвергают отсутствующий и неизвестный media type.
3. Ticket tests возвращают media type без Blob/provider authority.
4. Frontend tests удаляют active content и блокируют remote image requests.
5. Browser proof показывает HTML body внутри Mail viewer без внешнего браузера.
