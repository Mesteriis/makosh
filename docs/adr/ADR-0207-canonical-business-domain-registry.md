# ADR-0207: Канонический реестр бизнес-доменов Макошь

Статус: Принято
Дата: 2026-07-15
Состояние реализации: Не реализовано

Зависит от:

- [ADR-0200: Модульная модель и изоляция runtime](ADR-0200-clean-room-module-model-and-runtime-isolation.md);
- [ADR-0201: Взаимодействие ядра и модулей через IPC и NATS](ADR-0201-core-module-communication-and-nats.md);
- [ADR-0202: PostgreSQL, изоляция данных и PgBouncer](ADR-0202-postgresql-ownership-pgbouncer-and-extensions.md);
- [ADR-0204: Встроенные integration-плагины и нейтральная граница контекста](ADR-0204-bundled-integration-plugins-and-provider-neutral-context-boundary.md).

Implementation scope ограничен
[ADR-0208: Allowlist разработки доменов и запрет проекций](ADR-0208-domain-development-allowlist-and-projection-freeze.md).

## Контекст

Clean-room реализация требует закрытого начального списка владельцев durable
business truth до создания production packages, schemas и публичных
контрактов. Без такого списка один факт может получить нескольких владельцев,
integration может ошибочно стать business domain, а производная проекция —
начать хранить каноническую истину.

Особенно важно отделить Contacts от Organizations, provider-specific
operational experiences от Communications и канонические домены от Graph,
Timeline, Search и Context projections.

Этот ADR фиксирует только границы верхнего уровня. Внутренние сущности,
commands, queries, events и schema каждого домена определяются отдельно до
реализации соответствующего модуля.

## Решение

### Определение домена

**Бизнес-домен** — independently restartable module по ADR-0200, который
владеет одним bounded context, его публичным контрактом и durable business
truth. Один факт имеет одного канонического владельца.

Domain runtime не импортирует implementation или storage другого домена. Для
ссылок на объекты другого владельца используются public identifiers и
versioned contracts; для изменения чужого state — event и workflow с typed
command. Cross-domain SQL и foreign keys запрещены.

### Канонический список доменов

| `module_id` | Каноническое имя | Область владения |
|---|---|---|
| `communications` | Коммуникации | Provider-neutral communication evidence, provenance и его каноническая фиксация. Provider protocol, session и operational projection сюда не входят. |
| `contacts` | Контакты | Люди, их контактные данные и внешние идентичности. Организации этому домену не принадлежат. |
| `organizations` | Организации | Организации и их собственное durable состояние. Это отдельный домен, а не раздел Contacts. |
| `relationships` | Отношения | Подтверждённые отношения между субъектами с типом, временем и evidence. |
| `projects` | Проекты | Проекты, их состояние, ожидаемые результаты и жизненный цикл. |
| `tasks` | Задачи | Конкретные действия, их состояние, приоритеты и выполнение. |
| `obligations` | Обязательства | Зафиксированные обещания, договорённости и обязательства сторон. |
| `decisions` | Решения | Принятые решения, варианты, обоснование и связанное evidence. |
| `calendar` | Календарь | Календарные события, расписание и временные ограничения. |
| `documents` | Документы | Документы, их metadata, provenance и lifecycle. Blob bytes остаются за blob capability. |
| `knowledge` | Знания | Проверенное долговечное знание и его связь с evidence. |
| `review` | Обзор | Очередь разбора, review state и явное принятие, отклонение или продвижение предложений. |
| `ai` | AI | AI-конфигурация, агенты, запросы, запуски, provenance и lifecycle AI-операций. AI output не является business truth другого домена. |

Список закрыт для первой clean-room реализации. Добавление, объединение,
разделение или удаление бизнес-домена требует ADR, который изменяет или
заменяет настоящее решение.

### Что не является бизнес-доменом

#### Kernel и platform services

Kernel, PostgreSQL storage capability, PgBouncer, NATS, vault, blob storage,
clock и scheduler являются technical control/platform plane. Они не владеют
business truth.

#### Интеграции

Mail, Telegram, WhatsApp и Zulip являются встроенными integration-плагинами,
а не бизнес-доменами. Каждый владеет своим protocol, auth/session runtime,
cursor, provider-specific operational state и frontend experience. На границе
контекста integration публикует provider-neutral evidence для Communications.

Новый communication provider добавляется как integration module и не
расширяет список бизнес-доменов. Конкретные model runtimes и remote AI
providers являются adapters/integrations домена AI, а не самостоятельными
business domains.

#### Workflows

Workflow координирует публичные контракты нескольких доменов, но не становится
владельцем их state и не создаёт собственную копию business truth.

#### Производные проекции

Graph, Timeline, Search и Context являются rebuildable projections или engine
capabilities. Они могут иметь отдельные runtime и storage для derived state,
но не владеют каноническими Contacts, Organizations, Relationships, Projects
или другими domain entities.

`Memory` является продуктовой способностью Макошь, формируемой из evidence,
knowledge и domain state, а не отдельным универсальным владельцем всех данных.

### Ownership правила

- Organizations всегда остаётся отдельным владельцем от Contacts.
- Communications не поглощает provider operational models.
- Contacts и Organizations не импортируют друг друга; их координация идёт
  через contracts и workflows.
- Relationships хранит собственные relationship facts, но не копирует
  каноническое состояние участников.
- Review владеет решением о разборе и продвижении, но target entity создаёт
  только её целевой domain.
- AI владеет AI-конфигурацией и lifecycle запусков, но не создаёт и не изменяет
  business truth другого domain напрямую. Его результат является proposal,
  classification, summary, extraction или другим typed output с provenance.
- Graph, Timeline, Search и Context могут быть полностью пересозданы из owner
  events и canonical state.
- Provider identity разрешена в provenance, но не определяет business behavior
  домена.

## Отклонённые варианты

### Organizations внутри Contacts

Отклонено: у организаций собственные идентичность, lifecycle и причины
изменения. Объединение создаёт размытый owner и усложняет независимое развитие
обоих bounded context.

### Каждый provider как business domain

Отклонено: Mail, Telegram, WhatsApp и Zulip являются каналами и operational
experiences, но не владельцами проектов, задач, отношений, знаний или других
business facts.

### Graph как канонический владелец отношений и объектов

Отклонено: Graph является производным представлением. Истина остаётся у
Relationships и остальных domain owners.

### Один общий Context или Memory domain

Отклонено: такой модуль стал бы новым монолитом, копирующим state всех
остальных владельцев. Context и memory собираются через projections, evidence
и явные workflows.

### AI как скрытая platform utility

Отклонено: AI имеет собственные конфигурацию, policy, agents, runs, provenance
и failure lifecycle. При этом статус бизнес-домена не даёт AI права мутировать
state других owners.

## Последствия

Положительные:

- у каждого вида durable business truth появляется один владелец;
- Organizations развивается и перезапускается независимо от Contacts;
- AI runtime и его state имеют явного владельца;
- integration failures не смешиваются с отказами business domains;
- производные механизмы можно перестраивать без потери canonical state;
- package, schema role, NATS permissions и client contracts можно проверять
  против одного реестра.

Отрицательные:

- cross-domain сценарии требуют explicit workflows и versioned contracts;
- интерфейсу нужны read compositions вместо прямых cross-domain joins;
- внутренний глоссарий и contracts каждого домена ещё должны быть приняты
  отдельно до реализации.

## Проверка решения

До признания решения реализованным должны существовать:

- executable inventory с ровно перечисленными domain `module_id`;
- отдельные `ModuleDescriptorV1`, contract package, runtime identity и PostgreSQL role для
  каждого реализованного домена;
- dependency guard, запрещающий domain-to-domain implementation imports;
- storage guard, запрещающий cross-domain SQL и foreign keys;
- tests, доказывающие, что отказ одного domain runtime не останавливает Kernel
  и другие domains;
- negative checks, запрещающие Graph, Timeline, Search и Context как
  canonical domain owners;
- negative checks, запрещающие Mail, Telegram, WhatsApp и Zulip как domain
  roles;
- AI contract tests, запрещающие AI output напрямую мутировать state другого
  domain.

До появления этих executable evidence поле `Состояние реализации` остаётся
`Не реализовано`.
