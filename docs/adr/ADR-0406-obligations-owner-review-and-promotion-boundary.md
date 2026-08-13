# ADR-0406: Obligations owner, Review candidate and promotion boundary

Статус: Принято
Дата: 2026-08-13
Состояние реализации: В разработке

Зависит от ADR-0200, ADR-0201, ADR-0202, ADR-0207, ADR-0208 и ADR-0405.

## Контекст

`obligations` зарегистрирован как единственный владелец зафиксированных
обещаний, договорённостей и обязательств сторон, но до этого решения оставался
заблокированным. Tasks не может хранить promise truth, Calendar — обязательство
вместе со сроком, Communications — извлечённое обещание, а Review — состояние
целевого обязательства.

Потенциальное обязательство из communication/evidence нельзя сразу превращать
в business truth. Нужны отдельная Review-owned очередь и явный promotion
workflow, который после решения пользователя вызывает публичную команду
Obligations и коррелирует terminal result обратно в Review.

## Решение

### Owner boundary

`obligations` владеет подтверждённым обязательством, его statement, сторонами,
условием, сроком, lifecycle, evidence links и revision. Стороны и evidence
используют только typed public owner/record IDs и digest; чужое canonical state
не копируется и cross-domain SQL/FK запрещены.

Lifecycle закрыт состояниями Open, Fulfilled, Waived, Breached и Cancelled.
Due date может находиться в будущем. Terminal obligation не изменяется обратно
в Open скрытым переходом; re-open, если понадобится, требует отдельной явной
команды и нового решения.

### Review and promotion

`review-obligation-candidate` — Review-owned aggregate с Pending, Approved,
Rejected, Promoting, Promoted и PromotionFailed lifecycle. Candidate хранит
bounded proposed statement/condition, typed public party IDs, due date и
evidence digest. Candidate content передаётся через owner-authorized Blob
custody, а публичные события не содержат private statement/condition.

`reviewed-obligation-candidate-promotion` — отдельный workflow. Он потребляет
только exact Approved candidate, проверяет candidate/action digest, публикует
одну idempotent reviewed-candidate command в Obligations, связывает terminal
result и публикует typed promotion result обратно Review. Unknown terminal
ACK-ignore, exact replay возвращает сохранённые bytes, mismatch конфликтует.
Review runtime не изменяет Obligations напрямую, а Obligations не принимает
неподтверждённый candidate event.

### Storage and privacy

Все три владельца имеют отдельные owner-local PostgreSQL bundles с ENABLE и
FORCE RLS, tx-local logical owner и effective NOBYPASSRLS evidence. Inbox,
operations and outbox exact-bind canonical bytes/SHA. Relay использует
owner-local sequence и transaction-held `FOR UPDATE SKIP LOCKED` claim через
broker acknowledgement.

Запрещены provider payload, communication body, credential, private locator,
arbitrary JSON, copied foreign record, confidence/risk score и автоматическая
promotion. Public events содержат только IDs, revisions, closed states,
temporal fields и digests.

### Client and product surface

Obligations получает generated Connect client и отдельную compiled surface для
confirmed owner truth. Universal Review product композирует новый typed
candidate client. UI не использует REST alias/fallback и не отображает Graph,
Timeline, inferred risk или generic metadata.

## Изменение allowlist

Этим ADR `obligations` переносится из `domains.blocked` в
`domains.developmentAllowlist`. `decisions` остаётся единственным blocked
business owner. Это разрешает repository-local implementation, но не меняет
production `currentSlice`: admission Tasks 10–21 остаётся последовательным.

## Проверка

- exact API/core lifecycle и digest tests;
- FORCE-RLS bundles и effective NOBYPASSRLS matrices;
- actual candidate submit/approve/reject, promotion, target terminal, replay,
  restart, outage and privacy contours;
- generated Obligations and Review clients without REST fallback;
- compiler-consumed artifacts for all three modules;
- exact package/capability/inventory guards;
- full pre-push only at an authorized sequential production boundary.

## Не входит

- automatic obligation extraction or AI auto-approval;
- Decisions, Risk, Graph, Timeline, Search or Context;
- cross-owner SQL/FK or generic review item;
- compatibility route, branch, worktree, stage, commit or push.
