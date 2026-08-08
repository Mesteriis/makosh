# ADR-0397: Mail RFC822 decoding and canonical body view composition

- Статус: принято
- Дата: 2026-08-04
- Состояние реализации: реализовано для RFC 2047 headers, MIME transfer/charset
  decoding, HTML-only readable fallback и app-level canonical body composition.
- Связанные решения: ADR-0204, ADR-0205, ADR-0245, ADR-0257, ADR-0315,
  ADR-0325

## Контекст

Mail operational projection показывала сырой RFC822 multipart body как
`snippet`, когда письмо не имело подходящей UTF-8 `text/plain` части. Encoded
words в Subject/From не декодировались. Presentation пыталась показывать этот
bounded fallback как тело письма, хотя полный body принадлежит canonical
Communications content boundary.

CSS или клиентский parser не исправляют источник: raw MIME уже был ошибочно
materialized как operational text. Копирование body в Mail PostgreSQL также
создало бы вторую plaintext authority и нарушило ADR-0315.

## Решение

`hermes-mail-core` является единственной build unit для bounded RFC822/MIME
decoding, общей для IMAP и Gmail:

- RFC 2047 `B`/`Q` encoded words декодируются до operational Subject/From/To;
- base64 и quoted-printable декодируются до bounded bytes;
- declared charset преобразуется в UTF-8 с bounded legacy fallback;
- `text/plain` предпочитается; HTML-only письмо даёт только видимый inert text;
- malformed/oversized MIME fail closed и никогда не становится snippet/body;
- raw MIME и HTML не входят в Mail operational query, logs или errors.

IMAP сохраняет owner-configured total sync window, но finalizes fetched data
страницами не более 20 сообщений. Это отделяет provider scan bound от durable
page-finalization latency: первые актуальные письма и их body observations
становятся видимыми до завершения большого historical window. Выбранные UID
обходятся latest-first; historical completeness сохраняется в том же bounded
окне, но старые письма больше не блокируют актуальный список.
Перед latest-first хвостом Mail передаёт IMAP adapter bounded набор уже известных
INBOX UID из текущей operational projection. Это repair-приоритет, а не новая
authority: серверный `UID SEARCH` подтверждает существование каждого UID,
дубликаты удаляются, общий owner-configured sync limit не расширяется.

Основной operational message list сортируется по provider `sent_at` от новых
писем к старым, а не по времени обновления локальной проекции. Недатированные
записи идут после датированных; `cursor_sequence DESC` остаётся стабильным
tie-breaker. Cursor prefix `m2m` отделяет этот порядок от прежних `m1m` cursors,
чтобы страница, начатая в старой семантике, не продолжалась в новой.

Полное тело не копируется в Mail integration. Application composition делает:

```text
Mail observation_anchor_id
  -> Communications metadata query (provider-neutral evidence -> message_id)
  -> communications.content.v1 one-use ticket
  -> authenticated client_blob read
  -> fatal UTF-8 decode
  -> inert HtmlPreview text rendering
```

Mail integration не импортирует Communications domain/API. Cross-owner
composition находится в `frontend/src/workflows/mail-message-content`, а app
передаёт integration presentation только display state и decoded text.
Communications `GetEvidence` возвращает optional canonical `message_id`; он не
возвращает provider locator, body, Blob reference или credentials.

## UI disposition

Reference Mail viewer переносится по ответственности: envelope/actions остаются
Mail operational UI, message paper использует shared `HtmlPreview`, loading
показывает skeleton, а недоступный canonical content показывает typed state.
Snippet остаётся только list preview и не подменяет успешно загруженное body.

Rendered provider HTML, remote images и inline CID resources в этот revision не
входят: ADR-0315 допускает только exact admitted UTF-8 body bytes. Их нельзя
имитировать через `v-html`; отдельное расширение content media contract должно
сначала определить sanitization, remote-image proxy и Blob custody.

## Проверка

Gate требует:

1. RFC 2047, quoted-printable/base64, UTF-8 и legacy charset unit tests;
2. HTML-only regression, доказывающий отсутствие raw MIME в preview;
3. provider-evidence -> canonical-message resolution без provider locator;
4. workflow test для one-use content read и unavailable capability;
5. frontend typecheck/build и live browser proof на server-synced Mail data.
