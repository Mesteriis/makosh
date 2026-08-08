# ADR-0312: Mail permanent delete confirmation and provider authority

- Статус: принято
- Дата: 2026-07-28
- Состояние реализации: implemented. Exact command/query contracts, Mail
  storage bundle V17, provider adapters, managed runtime, generated frontend
  clients, explicit destructive confirmation и managed conformance открыты
  атомарно как `mail_message_permanent_delete_command_v1`.
- Связанные решения: ADR-0204, ADR-0205, ADR-0212, ADR-0213, ADR-0215,
  ADR-0220, ADR-0223, ADR-0253, ADR-0278, ADR-0298, ADR-0307, ADR-0308

## Контекст

Reversible Mail location mutations уже имеют отдельный
`mail.message-location.command.v1`: archive, trash, restore и move. Historical
backend намеренно не выполнял provider-side physical deletion. В clean-room
inventory необратимое удаление остаётся отдельным обязательным gate, потому
что оно имеет другие authority, confirmation, provider scope, retry и
projection semantics.

Permanent delete нельзя добавить как ещё один location kind:

- случайный повтор обычной location-команды не должен уничтожать письмо;
- Communications не владеет provider mailbox и не вызывает Mail runtime;
- Kernel/Core Gateway не интерпретируют Mail destructive policy;
- Gmail `gmail.modify` не даёт authority для physical delete;
- обычный IMAP `EXPUNGE` может удалить другие pending `\Deleted` messages;
- optimistic removal из UI до provider confirmation создаёт fake state;
- удаление Mail operational projection не должно уничтожать canonical
  Communications evidence.

## Решение

### Exact public contracts

Mail integration получает два независимых client contracts:

```text
mail.message-permanent-delete.command.v1
mail.message-permanent-delete.query.v1
```

Typed command содержит только:

- bounded non-empty `operation_id`;
- exact `connection_id`;
- stable Mail-owned `message_id`;
- `expected_projection_revision`;
- typed `PERMANENT_DELETE_CONFIRMED` confirmation.

Generic action string, provider locator, mailbox name, Gmail label, credential,
message body и Communications identity запрещены. Query возвращает только
sanitized durable status:

```text
pending
succeeded
rejected
unsupported
reauthorization_required
outcome_unknown
```

Command и query имеют разные capabilities, routes и grants. Accepted receipt
не означает provider completion.

### Destructive preconditions

Permanent delete разрешён только когда текущая Mail projection одновременно:

1. существует в той же `connection_id`;
2. имеет exact `expected_projection_revision`;
3. находится в ровно одной provider-discovered folder роли `trash`;
4. имеет current provider locator/binding;
5. исполняется current runtime с current grant, storage и credential
   generations.

Пользователь всегда сначала выполняет reversible trash. Stale UI, message
outside Trash, missing or ambiguous Trash role и cross-account identity
отклоняются до provider I/O.

Frontend показывает отдельный destructive confirmation control. Presentation
не конструирует provider command и не удаляет message optimistically. Typed
confirmation формирует Mail permanent-delete controller только после явного
пользовательского действия.

### Durable acceptance and replay

Mail persistence до provider I/O сохраняет exact canonical command bytes и
SHA-256 в отдельном owner-local journal. Повтор `operation_id` с теми же bytes
возвращает исходный receipt; conflicting bytes отклоняются.

Pending record остаётся replayable после restart. Runtime повторно декодирует
exact bytes и проверяет identity/revision/fences перед каждой provider attempt.
Provider execution задаёт convergent desired state "exact message отсутствует".
Definite rejection не повторяется; transport outcome без доказательства
становится `outcome_unknown`.

### IMAP authority

IMAP permanent delete:

1. загружает private `mailbox_id / UIDVALIDITY / UID` locator;
2. требует, чтобы mailbox имел current exact role `trash`;
3. требует advertised `UIDPLUS`;
4. выполняет read-write `SELECT` exact mailbox;
5. сверяет `UIDVALIDITY`;
6. выполняет `UID STORE <uid> +FLAGS.SILENT (\Deleted)`;
7. выполняет только `UID EXPUNGE <uid>`.

Обычный `EXPUNGE`, sequence number, wildcard UID, copy/delete fallback и
hardcoded Trash name запрещены. Missing `UIDPLUS` получает `unsupported`.

### Gmail authority and explicit reauthorization

Existing operational OAuth запрашивает fixed `gmail.modify` и `gmail.send`.
Он не считается authority для permanent delete.

Existing Gmail OAuth lifecycle получает typed authority:

```text
OPERATIONAL
PERMANENT_DELETE
```

`PERMANENT_DELETE` строит отдельный consent URL с exact
`https://mail.google.com/` scope. Authorization attempt сохраняет requested
authority; completion fail closed проверяет returned granted scope и сохраняет
sanitized `permanent_delete_authorized` binding. Scope string и tokens не
попадают в Settings, logs, errors, client status или health.

Обычный setup не запрашивает broad scope. Upgrade не происходит автоматически
при runtime restart, settings apply или первом delete. Без отдельного
успешного consent команда завершается `reauthorization_required` до Gmail API.

Gmail execution использует exact
`DELETE /gmail/v1/users/me/messages/{provider_message_id}`. `204` является
provider success; exact `404` при retry считается convergent already-absent
success. Другие provider statuses классифицируются без private response body.

### Atomic projection reconciliation

После provider success одна Mail-owned PostgreSQL transaction:

- переводит permanent-delete operation в terminal success;
- удаляет private provider locator;
- удаляет Mail operational message и folder memberships;
- пересчитывает affected Mail folder/thread counts and revisions;
- сохраняет monotonic deletion projection revision в operation status.

Mail operational projection является rebuildable provider experience, поэтому
удаление её row допустимо. Communications canonical evidence, observation
anchor history, Blob custody и source provenance не удаляются и не изменяются.
Mail не вызывает Communications store/query. Если отдельное provider-observed
delete evidence будет добавлено позже, оно проходит только существующий typed
integration outbox/event ingress, а не direct domain call.

### Core agreement and build units

```text
makosh-mail-api
  exact delete/query contracts and Gmail OAuth authority enum

makosh-mail-core
  pure destructive precondition and OAuth-scope decisions

makosh-mail-imap
  UIDPLUS + exact UID EXPUNGE adapter

makosh-mail-gmail
  authority-specific consent URL and exact DELETE adapter

makosh-mail-persistence
  OAuth authority binding, delete journal and atomic projection removal

makosh-mail-runtime
  current-fence orchestration and provider outcome classification

frontend Mail integration
  generated clients, destructive controller and confirmation presentation
```

Core Gateway authenticates client, resolves exact capability/contract digest,
runtime generation and grant epoch, then transports opaque bytes. Kernel does
not import Mail packages, decode confirmation, inspect OAuth scope or become a
generic deletion service. Communications does not import Mail and receives no
direct call.

### Gate `mail_message_permanent_delete_command_v1`

Gate становится `implemented` только атомарно при наличии:

1. independent exact command/query contracts and capability grants;
2. bounded typed confirmation and stale projection-revision fence;
3. owner-local exact-byte journal and restart-safe replay;
4. Trash-only current-location precondition;
5. IMAP `UIDPLUS` + exact `UID EXPUNGE` without ordinary `EXPUNGE`;
6. Gmail explicit broad-scope reauthorization and exact DELETE;
7. atomic operation, locator, message, thread and folder reconciliation;
8. no deletion of Communications evidence or cross-owner calls/SQL;
9. generated frontend client, SRP controller and destructive confirmation;
10. managed positive/replay, stale-revision, missing-scope and unsupported
    provider conformance;
11. architecture/Cargo guards preserving integration, domain and build-unit
    boundaries;
12. observability free of credentials, OAuth carriers, provider payloads and
    private message content.

Only after all items pass may `mail_operational_command_v1` close.

Gate закрыт вместе с `mail_operational_command_v1`. Managed evidence:

- `managed_mail_message_permanent_delete_is_fenced_exact_and_replay_safe`
  подтверждает stale-revision rejection до provider I/O, missing `UIDPLUS`,
  exact `UID EXPUNGE`, provider success, local projection removal и replay;
- `managed_mail_gmail_oauth_rotates_credentials_once_and_fails_closed`
  подтверждает отдельный permanent-delete consent URL и fail-closed rejection
  under-scoped provider response без ротации credential binding.

## Последствия

Permanent delete becomes an explicit high-authority Mail operation rather than
an accidental extension of reversible location state. Gmail owners consciously
grant broad scope, IMAP cannot expunge unrelated messages, stale UI cannot
delete a newer projection, and canonical Communications evidence survives the
provider-side destructive action.
