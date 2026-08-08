# ADR-0382: Mail address-book provider execution and authority

Статус: Принято

Дата: 2026-08-02

Состояние реализации: provider execution slice частично закрыт live evidence.
Reverse upsert и
provider-to-Contacts pagination имеют отдельные consumers/workers, provider
units, typed Settings/OAuth authority, Mail-owned atomic inbox/outbox, opaque
cursors и exact terminal events. Target-bound Blob custody/read, Google People
create/update и iCloud read-only rejection подключены к Mail managed runtime.
Target custody receipt сохраняется в Mail-owned PostgreSQL до Blob read и
provider dispatch; disposable PostgreSQL conformance доказывает restart replay,
идемпотентную запись и conflict rejection. Signed managed Mail process теперь
проходит отдельный disposable Vault/Storage/PgBouncer/NATS conformance с
реальными loopback TLS dialects Google People и CardDAV: отдельные typed
endpoints, отдельный CardDAV Vault purpose, provider read, exact
observation/result, duplicate suppression и successor Google runtime generation
доказаны. Signed managed ensemble теперь выполняет reverse target-bound Blob
custody и Google People update с
ETag, а workflow принимает exact terminal Mail result и завершает связанный
bidirectional run. Отдельный disposable PostgreSQL test доказывает atomic
terminal commit после нового connection, replay и hash conflict fencing.
Explicit iCloud read-only и missing-scope negatives закрыты signed managed Mail
flows с typed terminal results через NATS и нулевым Blob/provider IO. CardDAV
credential binding и lifecycle state хранятся в отдельных additive Mail-owned
tables, а не в IMAP binding. Google create теперь доказан signed managed
ensemble: returned provider ID/ETag event-only закрепляются Contacts-owned
command, а следующее изменение выполняет ETag-fenced PATCH вместо повторного
create. Managed outage/recovery/revoke conformance закрыт: NATS outage не
вызывает provider IO, post-write response loss даёт `OUTCOME_UNKNOWN` без
автоповтора, а recovery закрепляет наблюдённый ETag через Contacts-owned
command. Browser conformance ещё не закрыт, поэтому наличие этого среза не
открывает `mail_contacts_sync_v1`.

Уточняет:

- [ADR-0204](ADR-0204-bundled-integration-plugins-and-provider-neutral-context-boundary.md);
- [ADR-0222](ADR-0222-kernel-settings-registry-and-supervised-reconfiguration.md);
- [ADR-0223](ADR-0223-encrypted-sqlite-vault-and-scoped-credential-leases.md);
- [ADR-0379](ADR-0379-mail-address-book-sync-and-contacts-command-boundary.md);
- [ADR-0381](ADR-0381-contacts-target-bound-mail-sync-source-port.md);
- [ADR-0383](ADR-0383-contacts-provider-link-reconciliation-after-mail-write.md).

## Контекст

Mail runtime уже различает Gmail и IMAP для почтового транспорта, но это не
определяет address-book provider. IMAP hostname не является authority для
выбора CardDAV, Gmail API endpoint не является Google People endpoint, а
`mail_imap_password` нельзя переиспользовать для CardDAV с другим purpose.

Contacts-to-Mail command содержит только Mail account и target-bound Blob
receipt. Это намеренно: Contacts domain и workflow не должны выбирать provider,
видеть credential или импортировать provider adapter.

## Решение

Mail integration получает две отдельные provider implementation units:

- `makosh-mail-google-people` — bounded Google People list/create/update HTTP
  adapter без persistence, Vault, Event Hub или Contacts imports;
- `makosh-mail-carddav` — bounded CardDAV list adapter. Remote write для iCloud
  в первом gate запрещён и возвращает typed `READ_ONLY_PROVIDER` до provider
  mutation.

Mail Settings schema явно задаёт для configuration instance один
`address_book_provider`: `none`, `google_people` или `icloud_carddav`. Runtime
валидирует совместимость с account transport и никогда не выводит provider из
hostname, email suffix или payload события. Provider endpoint configuration
остаётся typed и production-pinned; loopback endpoints разрешены только в
conformance build.

Google Contacts write требует OAuth scope
`https://www.googleapis.com/auth/contacts`. OAuth completion сохраняет только
bounded boolean authority и scope digest; raw scope не хранится. Existing
bindings fail closed до повторной authorization. CardDAV получает отдельный
Vault purpose `mail_icloud_carddav_password`; IMAP credential для него не
переиспользуется.

Owner-local package `makosh-mail-address-book-persistence` добавляет exact
successor к `mail_state` после retained-replay revisions. Он владеет только
address-book inbox/job/result-outbox и replay fencing. Он не читает Contacts или
workflow tables. Mail runtime:

1. durable-резервирует exact upsert command до Ack и любых Blob/provider side
   effects; после этой owner-local reservation повторная доставка безопасна;
2. выбирает account и provider только из effective Mail Settings;
3. принимает target-bound Blob custody и сохраняет exact target reference/hash
   отдельным successor migration до чтения private bytes;
4. после restart читает уже сохранённый target receipt, проверяет declared
   size/hash и декодирует exact Contacts source content без повторного transfer;
5. выполняет provider mutation с ETag fencing;
6. атомарно сохраняет terminal result в Mail-owned outbox до его публикации;
7. при ambiguous provider outcome выдаёт `OUTCOME_UNKNOWN`, не повторяя mutation
   автоматически.

## Инварианты

- Mail integration не импортирует Contacts implementation или storage;
- workflow не читает Blob и не импортирует provider adapters;
- provider adapters не импортируют runtime, persistence, domains или Event Hub;
- private contact fields, provider entry IDs и credentials запрещены в
  subjects, logs, health и sanitized errors;
- completed replay публикует сохранённый exact result без Blob read или provider
  mutation;
- pending replay после завершённого custody использует сохранённый target
  receipt; partial receipt row и другой receipt для того же command fail closed;
- Google create/update различаются только по optional target-account link из
  source Blob; ETag передаётся как provider precondition;
- iCloud write и missing Google Contacts scope являются definite typed
  rejections;
- transport failure после provider request считается outcome-unknown.

## Gate

Static execution slice требует двух provider units, typed Settings и OAuth
authority, Mail storage successor, event consumer/result relay, Blob
receipt/hash negatives и architecture/Cargo/unit/PostgreSQL tests. Полный gate
дополнительно требует real Google People and CardDAV conformance, restart,
redelivery, revoke/generation/grant, provider outage, shared SSE, browser and
pre-push evidence.

## Отклонённые варианты

### Добавить People методы в `makosh-mail-gmail`

Gmail mailbox transport и Google People address book имеют разные API,
endpoints, scopes и причины изменения.

### Выбирать CardDAV по IMAP hostname

Hostname является конфигурационной деталью транспорта, а не явной owner
authority; alias или proxy тихо изменил бы business behavior.

### Использовать IMAP password для CardDAV

Credential lease выдан для другого exact purpose и не даёт CardDAV authority.
