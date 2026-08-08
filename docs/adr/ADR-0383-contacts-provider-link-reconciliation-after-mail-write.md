# ADR-0383: Contacts provider-link reconciliation after Mail write

Статус: Принято

Дата: 2026-08-03

Состояние реализации: implemented для Google People create/update
provider-link reconciliation. Typed command/results, Contacts-owned atomic
inbox/link/outbox, workflow-owned reconciliation ledger, exact replay/conflict
fencing и signed managed `POST -> bind link -> PATCH` ensemble доказаны.
Managed outage/recovery/revoke gate также доказан; recovery обновляет
Contacts-owned provider ETag без повторной remote mutation.
`mail_contacts_sync_v1` остаётся planned только до browser conformance.

Уточняет:

- [ADR-0201](ADR-0201-core-module-communication-and-nats.md);
- [ADR-0204](ADR-0204-bundled-integration-plugins-and-provider-neutral-context-boundary.md);
- [ADR-0379](ADR-0379-mail-address-book-sync-and-contacts-command-boundary.md);
- [ADR-0381](ADR-0381-contacts-target-bound-mail-sync-source-port.md);
- [ADR-0382](ADR-0382-mail-address-book-provider-execution-and-authority.md).

## Контекст

Google People `createContact` возвращает новый provider entry ID и ETag только
после успешной remote mutation. До вызова Mail Contacts snapshot намеренно не
имеет target-account link. Если workflow завершит reverse operation сразу после
Mail result и не передаст созданную link обратно owner Contacts, следующее
изменение того же Contact снова выберет `createContact` и создаст provider
duplicate.

Mail integration не имеет права писать Contacts storage. Workflow также не
может читать Contacts SQL или превращать собственную projection в canonical
Contacts truth. Kernel, Gateway и Event Hub не интерпретируют provider link.

## Решение

Contacts command contract получает отдельную exact reconciliation family:

- `contacts_bind_mail_address_book_provider_link` command;
- `contacts_mail_address_book_provider_link_bound` terminal result;
- `contacts_bind_mail_address_book_provider_link_rejected` terminal result.

Команда содержит только logical owner, Contact ID и expected Contact revision,
Mail account ID, bounded provider kind, returned provider entry ID и optional
ETag. Credentials, raw provider payload, contact presentation fields и generic
metadata запрещены.

Поток после успешной Mail mutation:

```text
Mail EntryUpserted result
  -> mail_contacts_sync durable inbox
  -> Contacts-owned BindProviderLink command in workflow outbox
  -> NATS JetStream
  -> Contacts runtime
  -> atomic Contacts link mutation plus terminal result outbox
  -> NATS JetStream
  -> mail_contacts_sync terminal reconciliation
```

Contacts проверяет current Contact revision, отсутствие conflicting link у
Contact/account и отсутствие попытки привязать provider entry к другому
Contact. Exact replay возвращает сохранённый result; та же command identity с
другими bytes отклоняется. Link mutation не меняет presentation fields и не
публикует новый `ContactChangedForMailSyncV1`, поэтому feedback loop запрещён.

Workflow переводит reverse operation из provider-dispatch state в
`awaiting_contacts_link` и считает её завершённой только после Contacts terminal
result. Definite Contacts rejection завершает operation как rejected;
неопределённая доставка повторяется через owner-local outbox/inbox, но remote
provider mutation повторно не выполняется.

Для Google update используется тот же reconciliation command: он обновляет ETag
существующей exact link и тем самым сохраняет следующий write ETag-fenced. Для
Google create он впервые закрепляет returned entry ID. ICloud V1 до этой стадии
не доходит, потому что write read-only.

## Границы и единицы сборки

- `makosh-contacts-command-api` остаётся Contacts-owned public command unit и
  владеет только typed Mail-derived Contacts mutation contracts;
- Contacts core не импортирует Mail, workflow или integration implementation;
- Contacts persistence владеет link mutation, inbox и outbox transaction;
- Contacts runtime получает отдельный handler ответственности, а не ветвление
  внутри identity-upsert handler;
- `makosh-mail-contacts-sync-runtime` импортирует только public Contacts/Mail
  contracts и координирует terminal events;
- Mail integration не импортирует Contacts packages;
- Kernel/Core Gateway/Event Hub только допускают и маршрутизируют exact
  contracts, не содержат reconciliation business logic.

## Gate

Google create считается доказанным только если managed signed ensemble:

1. получает Contacts changed event без target-account link;
2. выполняет ровно один Google `POST createContact`;
3. принимает exact Mail terminal result;
4. event-only закрепляет returned provider link и ETag в Contacts;
5. повторное изменение выполняет `PATCH updateContact`, а не второй create;
6. переживает duplicate delivery/restart без повторной remote mutation;
7. проходит architecture, Cargo, PostgreSQL, managed runtime и full pre-push
   gates.

## Отклонённые варианты

### Разрешить Mail писать Contacts table

Нарушает owner storage и превращает integration в часть domain implementation.

### Хранить canonical link только в workflow

Workflow projection не является Contacts truth, а Contacts source snapshot не
может читать чужую storage.

### Считать успешный POST завершённым create gate

Это доказывает HTTP dialect, но оставляет повторный create и provider duplicate.
