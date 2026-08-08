# ADR-0376: Owner-authorized replay of retained durable evidence

Статус: Принято

Дата: 2026-08-02

Состояние реализации: protocol, producer-local persistence, exact-byte adapters,
durable delivery foundations и отдельный workflow API/core/persistence/runtime
component/assembly contour implemented; workflow managed runtime, producer
managed loops и development ensemble implemented. Managed happy-path gate
подтвердил expired-retention recovery до terminal Preview через один shared SSE
stream и client_blob. Managed negative/restart/privacy gate также подтверждает
already-consumed no-op, missing producer evidence, partial success, stale
generation, wrong human owner, реальную остановку/возврат NATS и restart
workflow generation без выдачи private payload. Полный pre-push gate прошёл,
включая workspace-wide clippy,
оба nextest-профиля, dependency/supply-chain gates, frontend unit/visual tests
и production build. Live browser проверка существующего safe attachment
подтвердила один SSE transport и accepted Preview request, но terminal
состояние не было достигнуто: generated frontend replay client и
provider-neutral acquisition exact producer selection пока отсутствуют.
Поэтому phase gate и inventory остаются `planned`. Отдельный workflow-owned
build unit `makosh-retained-evidence-replay-protocol` реализует bounded exact
message selection, producer registration, owner-device actor hash,
runtime/grant fences и sanitized terminal result без subject/query/payload
surface. Communications build unit
`makosh-communications-retained-evidence-replay-persistence` добавляет exact
index поверх собственного `communications_domain_outbox`, проверяет сохранённые
bytes/message/hash/contract и ведёт append-only replay audit как storage bundle
revision 17. Integration build unit
`makosh-mail-retained-evidence-replay-persistence` аналогично индексирует только
собственный `mail_attachment_security_outbox`, проверяет exact scan-candidate
contract и добавляет Mail storage successor revision 23. Producer-local publish
adapters реализованы поверх разных owner-specific command/result contracts:
`makosh-communications-retained-evidence-replay-contract` и
`makosh-mail-retained-evidence-replay-contract`; их wire schemas не импортируют
workflow protocol или реализацию другого owner. Оба adapters сверяют exact
registration/runtime/grant fences, получают original owner-local bytes,
фиксируют append-only audit и публикуют их без decode/re-encode и без изменения
исходного outbox publish state. Additive Communications revision 18 и Mail
revision 24 добавляют раздельные owner-local command inbox и terminal result
outbox: command ID/hash conflict проверяется до исполнения, а completed state и
exact result bytes сохраняются атомарно. Producer storage bundles также имеют
additive owner-local bounded scan ledgers: Communications revision
19 и Mail revision 25. Managed runtime loops последовательно индексируют только
собственные outbox records; каждый проверенный record отмечается в owner-local
scan ledger, поэтому посторонние lifecycle records не блокируют bounded scan, а
исходный outbox не изменяется. Communications допускает в replay index только
terminal `SafeForDelivery`, Mail — только exact scan-candidate observation.
Owner-specific contract units теперь
также строят exact workflow command и causally-bound terminal result envelopes.
Communications и Mail имеют раздельные durable consumer components: они
проверяют exact workflow source/capability/owner, повторно не исполняют completed
inbox, сохраняют terminal result до Ack, а NATS/Storage outage оставляют для
redelivery. Отдельные result relay components публикуют только сохранённые exact
bytes и отмечают только собственный result outbox. Workflow
`attachment_preview_evidence_replay` теперь имеет отдельные generated Start API,
pure coordination core, owner-local PostgreSQL operation/selection/command
outbox/result inbox, authenticated client component, два exact publisher и два
commit-before-Ack result consumer, descriptor и unsigned release assembly.
Client payload не принимает owner/device claims: они выводятся из
`ModuleClientRequestV1`. Workflow storage не читает Communications или Mail SQL.
Workflow executable реализует `serve-inherited`: проходит descriptor/settings
authentication, получает process-bound Vault storage lease и exact Event Hub
publish/subscribe permits, обслуживает authenticated client delivery, durable
command outbox и два commit-before-Ack result inbox. Communications и Mail
descriptors запрашивают только свои exact command-consume/result-publish routes;
их managed process loops используют отдельные owner-local replay persistence
build units, сохраняют terminal result до Ack и публикуют result outbox exact
bytes. Mail отдельно держит integration/storage owner и logical human owner;
replay-команда проверяется по human owner, а storage/Vault authority остаётся у
integration owner. Workflow runtime/storage assembly включён отдельной единицей
в signed development release и монотонный development module plan. Managed
conformance доказал восстановление после фактического истечения обоих source
subjects, два producer result, terminal Preview SSE и чтение через client_blob.
Он также обнаружил и закрыл гонку premature cancellation: workflow и Mail
больше не оборачивают bounded Event Hub pull собственным 25 ms timeout, который
мог оставить уже назначенное сообщение unacknowledged до JetStream redelivery.
Deadline принадлежит одному transport adapter; consumer отвечает только за
commit-before-Ack обработку.
Полный phase gate ещё не закрыт из-за browser replay acquisition gap.
Никакая SQL-правка исходного publish state не считается реализацией этого
решения.

## Контекст

JetStream является bounded transport, а не canonical archive. Новый consumer,
admitted после `max_age`, не может восстановить историческое evidence только из
broker. В Preview это проявилось на уже импортированных вложениях: canonical
projection и producer-owned durable evidence существуют, но Preview inbox пуст,
поэтому workflow остаётся в accepted state и не имеет права запрашивать bytes.

Автоматически увеличивать retention, читать чужие PostgreSQL tables из Preview,
сбрасывать `published_at` или строить Kernel replay facade нельзя. Эти варианты
нарушают owner authority, bounded resource policy либо exact-byte provenance.
ADR-0201 и ADR-0220 уже требуют explicit operator/owner operation для replay.

## Решение

Ввести platform-neutral replay protocol и отдельные owner-local adapters.
Replay не является domain, integration или Kernel capability:

- owner/device начинает typed operation через Core Gateway;
- operation выбирает exact producer registration, contract reference и bounded
  set canonical `message_id`; arbitrary subject, SQL predicate и read-all scope
  запрещены;
- Kernel только проверяет обычный route/grant/runtime fence и не читает payload;
- producer adapter сверяет owner-local outbox indexes, envelope SHA-256 и exact
  canonical bytes, затем публикует тот же byte buffer и тот же `message_id`;
- publish получает отдельный owner-local replay attempt/audit record. Canonical
  outbox row и исходный publish acknowledgement не переписываются;
- consumer применяет существующий inbox ID/hash contract. Уже обработанный факт
  становится no-op, новый consumer фиксирует его впервые;
- replay не меняет business truth и не создаёт provider command;
- если exact original bytes отсутствуют или contract требует transformed
  payload, replay запрещён. Migration создаёт новый typed envelope с новым
  `message_id` и causation на original message в отдельном ADR;
- secrets, provider session state, private body/blob bytes и raw payload не
  возвращаются в client response, logs, health, telemetry или audit summary.

Один общий replay service с доступом ко всем owner outbox запрещён. Каждый
producer сохраняет SRP: selection/authorization, exact-byte verification и
publish-attempt persistence являются отдельными owner-local build units либо
явно разделёнными components внутри уже admitted producer runtime. Integration
не становится domain, domain не импортирует integration, а replay между ними не
вводит direct call.

## Preview recovery slice

Для исторического Preview требуется отдельный
`attachment_preview_retained_evidence_replay_v1` gate:

1. Communications owner выбирает exact safety-event message для запрошенного
   attachment anchor без выдачи provider identity;
2. producer integration, владеющий exact scan-candidate outbox bytes, выбирает
   соответствующее observation по собственному owner-local index;
3. owner подтверждает оба bounded replay attempts одной use-case operation;
4. оба producer adapters публикуют только original exact bytes;
5. Preview получает facts обычными durable consumers, выполняет существующий
   order-independent join и продолжает custody/render/SSE flow;
6. отсутствие одного producer, original bytes либо owner proof даёт terminal
   sanitized unavailable result, а не бесконечный spinner и не fallback к
   чужому storage.

Координация exact producer operations принадлежит отдельному workflow. Он
хранит только operation/correlation state, не читает owner storage и не
импортирует domain/integration implementations. Его target commands и results
идут через durable events; client polling не вводится.

## Phase gate

Gate становится implemented только после:

1. versioned replay operation/result contracts и отдельной workflow assembly;
2. owner/device, registration, runtime generation и grant epoch fencing;
3. bounded exact message selection без arbitrary subject/query;
4. byte/hash/index verification и publish без decode/re-encode;
5. append-only replay audit и idempotent attempt replay;
6. new-consumer recovery и already-consumed no-op evidence;
7. expired retention, missing bytes, stale fence, wrong owner, partial producer,
   NATS outage/restart и privacy-negative conformance;
8. live Preview terminal SSE/client_blob browser proof без polling;
9. architecture, SRP, Cargo, frontend и full pre-push gates.

До выполнения gate ADR-0373 и inventory `attachment_preview_v1` остаются
`planned`.

## Отклонённые варианты

### Увеличить JetStream retention до бесконечности

Отклонено: broker не становится canonical archive и теряет bounded disk policy.

### Сбросить `published_at` в producer outbox

Отклонено: mutation скрывает replay attempt, меняет delivery evidence и требует
direct owner-storage intervention.

### Дать Preview доступ к Communications, integration или Attachment Security SQL

Отклонено: нарушает owner isolation и превращает workflow в facade.

### Реализовать replay в Kernel/Event Hub

Отклонено: Kernel не знает business selection и не получает generic payload или
owner-outbox authority.
