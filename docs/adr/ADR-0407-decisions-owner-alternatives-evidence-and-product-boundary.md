# ADR-0407: Decisions owner, alternatives, evidence and product boundary

Статус: Принято
Дата: 2026-08-13
Состояние реализации: В разработке

Зависит от ADR-0200, ADR-0201, ADR-0202, ADR-0207, ADR-0208 и ADR-0406.

## Контекст

`decisions` зарегистрирован как владелец принятых решений, рассмотренных
вариантов, обоснования и связанного evidence, но до этого решения оставался
единственным заблокированным business owner. Review decision — это решение о
candidate workflow, но не самостоятельная canonical Decision. Tasks,
Projects, Obligations и Communications также не могут владеть общей историей
принятого решения.

## Решение

### Owner boundary

`decisions` владеет bounded title/question, rationale, lifecycle, typed
alternatives, выбранным alternative ID, typed public evidence links и checked
revision. Alternative содержит stable ID, bounded title/description и
disposition Candidate/Selected/Rejected. Evidence использует только public
owner/record ID, revision и digest; чужое canonical состояние не копируется.

Lifecycle закрыт состояниями Draft, Decided, Superseded и Cancelled. Только
Draft можно редактировать, пополнять alternatives/evidence и переводить в
Decided или Cancelled. Decided требует не менее двух alternatives, ровно один
Selected и непустое rationale. Supersede разрешён только из Decided и требует
typed replacement Decision ID. Terminal records не переоткрываются скрытым
переходом.

### Persistence and privacy

Owner-local PostgreSQL bundle хранит decisions, alternatives, evidence, exact
client operations и sequenced public outbox. Все таблицы используют ENABLE и
FORCE RLS, tx-local logical owner и effective NOBYPASSRLS evidence. Operation
replay exact-bind canonical request/response bytes и SHA. Relay держит
`FOR UPDATE SKIP LOCKED` claim через broker acknowledgement.

Публичное lifecycle event содержит только Decision ID, revision, closed state,
chosen/replacement IDs и occurred_at. Title, rationale, alternative text и
evidence IDs остаются в owner client/storage boundary. Запрещены provider
payload, communication body, credential, arbitrary JSON, risk/confidence score
и cross-owner SQL/FK.

### Client and product surface

Generated Connect client предоставляет Create/Update, alternative/evidence
mutations, Decide/Supersede/Cancel и bounded Get/List/ListAlternatives/
ListEvidence. Отдельная compiled Decisions surface отображает canonical owner
truth. REST alias/fallback, Graph, Timeline, inferred risk и generic metadata
не создаются.

## Изменение allowlist

Этим ADR `decisions` переносится из `domains.blocked` в
`domains.developmentAllowlist`. Это завершает development freeze business
owners, но не меняет production `currentSlice`: admission Tasks 10–22 остаётся
последовательным.

## Проверка

- exact API/core lifecycle and mutation tests;
- FORCE-RLS bundle and effective owner-2 NOBYPASSRLS matrix;
- actual create/edit/alternatives/evidence/decide/supersede/cancel, replay,
  restart and privacy contour;
- generated client without REST fallback;
- compiler-consumed runtime and Storage artifacts;
- exact package/capability/inventory guards;
- full pre-push only at an authorized sequential production boundary.

## Не входит

- Review candidate decisions, AI auto-decision or extraction;
- Projects/Tasks/Obligations state mutation;
- Risk, Graph, Timeline, Search, Context or generic JSON;
- compatibility route, branch, worktree, stage, commit or push.
