# ADR-0412: Frontend account lanes over shared client realtime

Статус: Принято
Дата: 2026-08-16
Состояние реализации: Частично реализовано. Один shared authenticated Gateway
SSE, общий provider/account lane scheduler и typed projection invalidations
подключены для Telegram, Mail, WhatsApp и Zulip. Account adapters применяют
provider-owned monotonic revision и отбрасывают duplicate/out-of-order frames.
Telegram frontend больше не опрашивает provider replay route таймером.
Account-local snapshots в Mail, WhatsApp и Zulip восстанавливаются до нового
запроса, сохраняют текущий selection и заменяются свежим projection атомарно.
Publication из runtime ограничена коротким control timeout и остаётся pending
при временной недоступности Gateway. Lane lifecycle, coalesced replay-gap
recovery и payload-safe browser Performance spans покрыты focused tests.
Shared browser fan-out изолирует исключение каждого observer: ошибка одного
provider/account adapter фиксируется только безопасным signal kind и не
прерывает доставку следующим logical lanes.

Telegram media materialization больше не использует один global active slot:
очередь имеет отдельный последовательный lane на account и общий bounded budget
из четырёх активных аккаунтов. Зависший файл account A не удерживает preview или
media account B, а interactive work выбирается раньше background work. История
чата накапливает provider pages поверх уже показанного snapshot; свежая страница
не теряется, даже когда она старше последних 500 durable rows.

Cached Telegram chats/messages обслуживаются из owner-local durable projection
даже пока TDLib повторно проходит authorization. Frontend показывает этот
read-only snapshot с явным состоянием offline authorization, но provider sync,
commands и realtime не объявляет доступными до `ready`.

Persistent WhatsApp host bridge больше не удерживает runtime actor в
блокирующем socket loop: отдельный bounded I/O worker передаёт actor не более
одного provider frame за tick, поэтому client delivery, realtime и outbox
продолжают обслуживаться тем же account runtime.

Zulip history hydration, event long-poll и outbound provider commands вынесены
из actor tick в отдельные bounded Tokio jobs. Jobs владеют только clone handle
к тому же owner-local PgPool, immutable provider configuration и точным
account lifecycle fence; control channel и mutable runtime state остаются у
actor. Blob grant и durable command claim происходят до handoff. Rebind или
retire повышает fence epoch, отменяет активный provider future и запрещает
устаревшему completion стать durable result. Process-root/runtime tests
удерживают jobs незавершёнными и подтверждают, что actor продолжает работу.

Multiplexed Mail runtime больше не выполняет send, message-flag, location и
permanent-delete provider I/O в catalog actor. Для каждого connection создаётся
отдельный bounded delivery job и один последовательный message-mutation lane
после короткой actor-owned prepare-фазы: durable claim, Vault/blob
materialization и создание immutable provider request. Последовательный lane
сохраняет порядок flag → location → permanent delete и не допускает гонку
перемещения с удалением одного сообщения. Account lifecycle, credential rebind
и успешная Gmail OAuth rotation повышают connection-local fence epoch; process
root отменяет stale job, а сам job повторно проверяет epoch до provider call и
перед durable completion.

Gmail OAuth, Gmail sync и IMAP sync также больше не используют один global
provider-operation slot на весь Mail catalog. Process root держит отдельный
bounded active slot по `connection_id`, поэтому provider I/O разных аккаунтов
исполняется параллельно, а claim/finalize и mutable catalog state остаются в
actor loop. Completion обязан совпасть с connection, под которым job был
запущен; mismatch завершается fail-closed. Deadline и cancellation применяются
отдельно к каждому connection, не останавливая sync соседнего аккаунта. Actor
принимает не более одной sync page от каждого connection за tick, поэтому один
быстрый producer не вычерпывает общий finalize loop раньше остальных аккаунтов.

Phase gate всё ещё не закрыт: replay-gap recovery и общий bounded-mailbox
overflow contract покрыты тестами, а WhatsApp persistent bridge и Mail
provider-command/account-local sync jobs имеют runtime-level и architecture
guards. Zulip history,
long-poll и command jobs имеют scheduler/fencing tests, но ещё нет live-provider
latency evidence. Также отсутствуют подключённый production telemetry sink,
live desktop evidence после пересборки и Android lifecycle/reconnect
conformance.
Telegram large-media source path реализован ADR-0413: TDLib-файл поступает в
receipt-bound encrypted chunks, а video/audio читаются через session-bound HTTP
range lease. Live playback/seek evidence после пересборки ещё не получено,
поэтому phase gate остаётся открытым.
Восстановленная Telegram session публикует `ready` до первого нового provider
update, а пустой provider tick ограничен короткой паузой и не крутит busy loop.

Уточняет:

- [ADR-0204: provider operational boundary](ADR-0204-bundled-integration-plugins-and-provider-neutral-context-boundary.md);
- [ADR-0205: Core Gateway and client transport](ADR-0205-core-gateway-and-client-transport.md);
- [ADR-0287: Telegram operational realtime replay](ADR-0287-telegram-operational-realtime-replay-boundary.md);
- [ADR-0290: Telegram account runtime](ADR-0290-telegram-account-runtime-reconfiguration-boundary.md);
- [ADR-0337: capability-routed managed client realtime](ADR-0337-capability-routed-managed-client-realtime.md).

## Контекст

Отдельный managed integration runtime изолирует provider failure от Kernel и
других modules, но сам по себе не гарантирует отзывчивость клиента. До этого
решения Telegram frontend сводил независимые операции в один последовательный
browser-local конвейер. Долгий provider lookup, media materialization или
replay poll поэтому мог задержать history query и следующий account action,
хотя Telegram уже исполнялся отдельным managed process.

Создание отдельного WebSocket для каждого provider account не исправляет эту
причину. Несколько sockets всё равно сходятся в один frontend scheduler и могут
использовать тот же блокирующий runtime request path. При этом каждый socket
дублирует authentication, reconnect, replay cursor, capability fencing и
Android lifecycle.

## Решение

### Один physical transport, независимые logical lanes

Frontend process использует один owner-authenticated multiplexed Gateway SSE:

```text
Core Gateway shared SSE
        ↓ typed contract demultiplexer
logical account lane (provider owner + account ID)
        ↓ bounded account mailbox
account controller + account-local projection store
```

Provider-specific WebSocket, второй SSE endpoint и прямое подключение клиента
к module runtime запрещены. Commands и snapshot queries остаются generated
ConnectRPC. Media bytes идут только через bounded Blob HTTP/range path.

Logical lane не является generic provider union. Общий demultiplexer проверяет
exact client-safe contract reference и передаёт typed payload зарегистрированному
frontend adapter. Только provider adapter декодирует account identity и
provider event semantics.

### Frontend account controller

Каждый открытый или фоново наблюдаемый account получает отдельный controller с:

- account-local projection store и applied revision;
- cancellation generation для chat/account navigation;
- bounded mailbox и stale/replay-gap state;
- независимыми budgets для snapshot, enrichment, preview и full media work;
- namespace для memory/file-backed media references;
- lifecycle `inactive -> hydrating -> live -> stale -> recovering -> closed`.

Один медленный account lane не удерживает применение events, history snapshot
или commands другого account. Переключение чата сначала показывает уже
имеющуюся account-local projection, затем запускает refresh. Provider network,
sender enrichment, preview materialization и full media download не входят в
критический путь первого render.

Frontend work разделяется минимум на четыре очереди:

1. `interactive` — navigation, cached history, composer и command receipt;
2. `realtime` — применение typed invalidation/transition frames;
3. `enrichment` — sender directory, context и derived presentation metadata;
4. `media` — preview и full blob materialization с отдельными bounded budgets.

`interactive` и `realtime` не ждут `enrichment` или `media`. Background work
кооперативно отменяется при смене generation. Новая foreground operation не
может ждать уже неактуальную background operation.

### Snapshot, replay и backpressure

Private bodies и media не передаются через SSE. Account lane получает typed
invalidation/transition, применяет её по monotonic revision/cursor и при
необходимости делает bounded owner query.

При переполнении account mailbox client не теряет события молча: lane становится
`stale`, отменяет производную background work, получает новый bounded snapshot
и продолжает с согласованного cursor. Остальные lanes не сбрасываются. Terminal
command result и replay gap никогда не coalesce-ятся; повторяемые invalidations
одного projection key могут быть объединены.

### Provider runtime responsiveness

OS-process topology остаётся provider-owned:

- Telegram сохраняет уже принятую account-scoped managed process topology;
- provider, для которого принят multiplexed runtime (например Mail), обязан
  иметь account-scoped actor/mailbox budgets внутри своего process;
- Kernel supervises runtimes и routes opaque contracts, но не создаёт
  provider/account sockets и не планирует provider jobs.

Внутри provider runtime realtime ingestion/provider IO не должен выполняться в
одном blocking critical section с client snapshot query. Cached operational
query читается из owner-local durable projection. Provider refresh, sender
resolution и media download являются bounded jobs и публикуют результат после
completion. Client request, который требует долгой provider network работы,
возвращает receipt/progress вместо удержания control channel.

### Observability

Каждая очередь публикует payload-safe spans/metrics:

- `lane_kind`, `work_class`, `queue_wait_ms`, `execution_ms`;
- queue depth, cancellation, stale transition и replay gap counts;
- runtime/control-route latency без account ID, chat ID, sender ID, body,
  filename, provider cursor или media bytes.

В developer mode разрешены полные typed field names и schema mismatch details,
но private values, credentials и provider payload остаются запрещены.

## Phase gate `frontend_account_lane_isolation_v1`

Gate реализован только при наличии:

1. одного authenticated Gateway SSE без provider/account WebSocket;
2. typed demultiplexing в независимые account controllers;
3. отсутствия provider replay polling timer во frontend;
4. snapshot-first chat/account navigation без ожидания provider IO;
5. раздельных interactive/realtime/enrichment/media budgets;
6. cross-lane test: stalled media/enrichment account A не задерживает history
   и realtime account B;
7. same-lane test: stalled media не задерживает новое message transition;
8. explicit overflow/stale/replay-gap recovery без silent loss;
9. provider-runtime test: long refresh job не блокирует cached query или
   provider update ingestion;
10. payload-safe queue/span diagnostics;
11. live desktop browser proof и Android lifecycle/reconnect conformance перед
    mobile admission.

## Отклонённые варианты

### WebSocket на каждый account

Отклонено: размножает transport state и не создаёт scheduler/process isolation.
Bidirectional low-latency transport остаётся допустим только для отдельно
доказанной calls/presence capability через новый ADR.

### Один global frontend promise queue

Отклонено: background provider/media operation создаёт head-of-line blocking
для navigation, realtime и другого account.

### Передавать bodies/media через shared realtime

Отклонено: нарушает privacy и bounded frame contract ADR-0205. Realtime несёт
только client-safe typed transition/invalidation; content читается отдельным
query/blob path.

### Process-per-account для каждого provider

Отклонено как общее правило: process topology принадлежит integration и зависит
от проверенной failure/security причины. Frontend account-lane isolation должна
работать одинаково и для account-scoped Telegram process, и для multiplexed
Mail runtime.
