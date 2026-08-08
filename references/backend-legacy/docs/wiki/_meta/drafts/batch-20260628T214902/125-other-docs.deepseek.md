### Summary / Резюме

Обновить страницу `components/docs.md` в русской Obsidian‑wiki на основе последних исходников статического документационного сайта Макошь (`docs/site/index.html` и `docs/site/makosh-docs.css`). Страница должна описать структуру страницы, доступные UI‑компоненты, систему стилей и зафиксированные в HTML‑файле тематические разделы (модель, точки входа, домены, движки, рабочие процессы, поверхность совместимости, планы рефакторинга), а также ссылки на внешние артефакты (спецификации, ADR и т. д.). Предыдущее содержимое страницы не встроено в контекстный пакет, поэтому предлагается полная замена.

### Proposed pages / Предлагаемые страницы

`components/docs.md`

```markdown
# Документационный сайт Макошь (docs)

## Обзор

Компонент `docs` — статический HTML‑сайт документации проекта Макошь, размещаемый в каталоге `docs/site/`.
Он служит единой точкой входа в спецификации продукта, модель памяти, глоссарий, доменные спецификации, движки, рабочие процессы, текущую реализацию и планы архитектурных изменений.

Исходный файл: `docs/site/index.html`.
Стили: `docs/site/makosh-docs.css`.
Логотип: `docs/site/assets/makosh-logo-mark.png` (бинарный файл размером 106 906 байт).

## Структура страницы

HTML‑документ построен как двухколоночная сетка с классом `docs-shell`:
- боковая панель `docs-sidebar` (левая колонка, фиксированная ширина 224 px);
- основная область `docs-main` (правая колонка, занимает оставшееся пространство).

На узких экранах сетка перестраивается в одну колонку.

## Боковая панель (`docs-sidebar`)

Панель содержит:

1. **Бренд** (`brand`):
   - ссылка‑якорь на `#top`;
   - изображение `assets/makosh-logo-mark.png` с классом `brand-mark` (32×32 px, содержит фильтр‑тень);
   - текстовая подпись `brand-copy`:
     - строка `Макошь` (жирный, размер 15 px, в верхнем регистре);
     - строка `Documentation` (размер 10 px, приглушённый цвет).

2. **Навигация** (`nav-group`):
   - семь ссылок‑якорей:
     - `#model` (активная при загрузке — `class="active"`) — «Model»;
     - `#entrypoints` — «Entrypoints»;
     - `#domains` — «Domains»;
     - `#engines` — «Engines»;
     - `#workflows` — «Workflows»;
     - `#implementation` — «Implementation»;
     - `#refactoring` — «Refactoring».

## Основная область (`docs-main`)

### Топ‑панель (`topbar`)

Содержит четыре метки (`span`):
- `Local-first`
- `Evidence-backed`
- `Event-sourced`
- `Persona-native target model`

Метки отображаются как акцентные «чипы» с зелёной рамкой и фоном.

### Вводная панель — Каноническая модель (`intro-panel`, id `model`)

- **Лейбл‑раздела** (`eyebrow`): *Canonical model*
- **Заголовок** (`h1`): *Personal Memory System*
- **Основной текст** (`lead`): *Макошь stores context about communications, knowledge, memory, relationships, projects, documents, decisions, obligations and the owner's operating context.*
- **Полоса потока** (`flow-strip`) с пятью метками в том же стиле:
  - `Communication`
  - `Source Evidence`
  - `Knowledge`
  - `Memory`
  - `Context`

### Сетка «Точки входа и Что НЕ является Макошь» (`grid two`, id `entrypoints`)

Две панели (`panel`):

1. **Canonical Entrypoints** (лейбл *Start here*):
   - Список ссылок (`link-list`) на файлы репозитория `https://github.com/Mesteriis/makosh-os/blob/main/`:
     - `docs/product/master-spec.md` — *Product Master Spec*
     - `docs/foundation/vision.md` — *Foundation Vision*
     - `docs/foundation/glossary.md` — *Glossary*
     - `docs/foundation/world-model.md` — *World Model*
     - `docs/README.md` — *Documentation Index*

2. **What Макошь Is Not** (лейбл *Current rule*):
   - Маркированный список (`plain-list`) из утверждений:
     - Email client
     - CRM or contact manager
     - Task tracker
     - Calendar app
     - Note-taking app

### Панель «Домены» (`panel`, id `domains`)

- Лейбл: *Durable entities*
- Заголовок: *Domains*
- Плиточная сетка (`tile-grid`) из десяти ссылок на спецификации доменов в репозитории:
  - `docs/domains/persons/README.md` — *Personas*
  - `docs/domains/communications/README.md` — *Communications*
  - `docs/domains/organizations/spec.md` — *Organizations*
  - `docs/domains/projects/README.md` — *Projects*
  - `docs/domains/documents/README.md` — *Documents*
  - `docs/domains/tasks/spec.md` — *Tasks*
  - `docs/domains/calendar/spec.md` — *Calendar and Events*
  - `docs/domains/decisions/README.md` — *Decisions*
  - `docs/domains/obligations/README.md` — *Obligations*
  - `docs/domains/graph/README.md` — *Knowledge Graph*

### Сетка «Движки и Рабочие процессы» (`grid two`)

Две панели:

1. **Engines** (лейбл *Derived mechanisms*):
   - Компактный список ссылок (`compact-links`) из восьми элементов:
     - `docs/engines/memory/README.md` — *Memory*
     - `docs/engines/timeline/README.md` — *Timeline*
     - `docs/engines/trust/README.md` — *Trust*
     - `docs/engines/search/README.md` — *Search*
     - `docs/engines/enrichment/README.md` — *Enrichment*
     - `docs/engines/obligation/README.md` — *Obligation*
     - `docs/engines/risk/README.md` — *Risk*
     - `docs/engines/consistency/README.md` — *Polygraph*

2. **Workflows** (лейбл *Evidence flow*):
   - Компактный список ссылок (`compact-links`) из шести элементов:
     - `docs/workflows/communication-to-knowledge.md` — *Communication to Knowledge*
     - `docs/workflows/communication-to-obligation.md` — *Communication to Obligation*
     - `docs/workflows/meeting-to-decisions.md` — *Meeting to Decisions*
     - `docs/workflows/document-to-context.md` — *Document to Context*
     - `docs/workflows/contradiction-review.md` — *Contradiction Review*
     - `docs/workflows/dossier-generation.md` — *Dossier Generation*

### Сетка «Реализация и Рефакторинг» (`grid two`)

Две панели:

1. **Compatibility Surface** (лейбл *Current implementation reality*):
   - Маркированный список (`plain-list`) из пяти утверждений:
     - `Active identity route: /api/v1/persons/{person_id}/identity`
     - `Historical contacts projection was renamed to persons.`
     - `Protected local APIs use X-Макошь-Secret.`
     - `New credentials use host vault storage.`
     - `Email channel code remains under current mail modules.`

2. **Refactoring Plans** (лейбл *Next work*):
   - Список ссылок (`link-list`) на планы рефакторинга и дорожную карту:
     - `docs/refactoring/completion-audit.md` — *Completion Audit*
     - `docs/refactoring/implementation-alignment-plan.md` — *Implementation Alignment Plan*
     - `docs/refactoring/product-alignment-plan.md` — *Product Alignment Plan*
     - `docs/product/development-roadmap.md` — *Development Roadmap*
     - `docs/adr/README.md` — *ADR Index*

## Система стилей (`makosh-docs.css`)

Стилевой файл определяет тёмную тему через CSS‑переменные на уровне `:root`:

- Цвета:
  - Фон: `#02090b` (`--hh-color-bg`), приподнятый `#020d10` (`--hh-color-bg-raised`).
  - Поверхности: `#06181b` (`--hh-color-surface`), глубокая `#041215` (`--hh-color-surface-deep`).
  - Текст: основной `#eefefb`, сильный `#f2fffd`, приглушённый `#91a8a8`, затемнённый `#849ca0`.
  - Акцентный циан: `#2df0ce` (`--hh-color-accent`), сильный `#25d8bd`.
- Рамки: несколько уровней прозрачности для акцента (`--hh-border-accent`, `--hh-border-accent-soft`) и общих разделителей (`--hh-border-subtle`).
- Тени: панели навигации (`--hh-shadow-sidebar`) и панелей (`--hh-shadow-panel`).
- Радиусы: основной `7px`, средний `8px`, круглый `50%`, для боковой панели `0 18px 34px 0`.
- Шрифты: системный стек без засечек (`Inter`, `SF Pro Display`, `ui-sans-serif`, …); моноширинный стек для `code` (`SFMono-Regular`, `Consolas`, …).

### Основные компоненты и их стилизация

- `.docs-sidebar`: фиксированное позиционирование (`sticky`), минимальная высота 100vh, полупрозрачный фон с размытием (`backdrop-filter: blur(12px)`), градиент, рамка и сложная тень.
- `.brand-mark`: размер 32×32 px, `object-fit: contain`, акцентная тень‑фильтр.
- `.brand-copy`: две строки с разным весом и размером шрифта, текст усекается эллипсисом.
- `.nav-group a`: минимальная высота 32 px, плавные переходы `180ms` для границы, фона, тени и цвета. Активное состояние выделяется рамкой, градиентным фоном и свечением.
- `.topbar`: флекс‑контейнер, метки внутри — «чипы» с акцентной рамкой и фоном, заглавным весом и размером 12 px.
- `.intro-panel`, `.panel`: панели с рамкой, внутренней тенью и градиентным фоном. Уводная панель имеет больший внутренний отступ (24 px против 18 px).
- `.eyebrow`: текст 11 px, жирный, в верхнем регистре, акцентного цвета.
- `h1` (заголовок первого уровня): максимальная ширина 760 px, размер 34 px, вес 760.
- `h2` (заголовок второго уровня): размер 18 px, вес 720.
- `.lead`: максимальная ширина 820 px, размер 15 px, межстрочный интервал 1.6.
- `.flow-strip`: флекс‑контейнер, метки идентичны `topbar span`, но могут иметь дополнительное позиционирование (`position: relative`).
- `.grid.two`: двухколоночная сетка с промежутком 16 px. Элементы внутри `.panel` занимают по одной колонке.
- `.link-list`, `.compact-links`, `.tile-grid`: сетки ссылок‑кнопок с одинаковым базовым стилем (рамка, приглушённый фон, текст цвета `#dff8f4`), меняющимся при наведении на акцентный. Различаются количеством колонок:
  - `.link-list` — одна колонка.
  - `.compact-links` — две колонки.
  - `.tile-grid` — пять колонок (широкие экраны), при ширине ≤ 1100 px — три колонки.
- `.plain-list`: маркированный список без маркеров, с левой акцентной границей 2 px и обычным текстом.
- `code`: моноширинный шрифт, акцентная рамка и фон.

### Адаптивность

Стили содержат три медиа‑запроса:

1. **max-width: 1100 px** — `.tile-grid` переходит в три колонки.
2. **max-width: 840 px** — `.docs-shell` становится одноколоночной; боковая панель теряет `position: sticky` и получает скругление `var(--hh-radius-md)`; `.nav-group` распределяется в две колонки; `.grid.two` становится одной колонкой.
3. **max-width: 620 px** — `.tile-grid`, `.compact-links` и `.nav-group` становятся одноколоночными; панели получают уменьшенный внутренний отступ (14 px); размер `h1` уменьшается до 26 px.

## Зависимости и ресурсы

- Внешних JavaScript‑зависимостей нет.
- Все внешние ссылки ведут на файлы в GitHub‑репозитории `Mesteriis/makosh-os` (ветка `main`).
- Иконки или веб‑шрифты не подключаются.
```

### Source coverage / Покрытие источников

- **`docs/site/index.html`** (8283 символов):
  - Структура HTML‑документа (оболочка, боковая панель, основная область).
  - Бренд‑логотип и ссылка‑якорь.
  - Навигационные ссылки и их идентификаторы.
  - Содержимое топ‑панели (четыре метки).
  - Вводная панель: заголовок, основной текст, полоса потока.
  - Панели с точками входа (канонические точки входа и список «Что НЕ является Макошь»).
  - Панель доменов со всеми десятью ссылками.
  - Панели движков и рабочих процессов со ссылками.
  - Панели совместимости и планов рефакторинга со списками и ссылками.
  - Все текстовые строки и атрибуты `href` сохранены буквально.

- **`docs/site/makosh-docs.css`** (7653 символов):
  - Полный набор CSS‑переменных (цвета, радиусы, тени, шрифты).
  - Стили для каждого HTML‑элемента и класса: `.docs-shell`, `.docs-sidebar`, `.brand`, `.brand-mark`, `.brand-copy`, `.nav-group`, `.nav-group a`, `.nav-group a.active`, `.docs-main`, `.topbar`, `.topbar span`, `.flow-strip span`, `.intro-panel`, `.panel`, `.eyebrow`, `h1`, `h2`, `.lead`, `.flow-strip`, `.grid`, `.grid.two`, `.link-list`, `.compact-links`, `.tile-grid`, `.plain-list`, `code`.
  - Правила адаптивности при ширине `1100px`, `840px`, `620px` и соответствующие изменения сеток, отступов и размеров шрифта.
  - Анимации и переходы (`transition`), используемые для интерактивных элементов.

- **`docs/site/assets/makosh-logo-mark.png`** (106906 байт):
  - Факт наличия файла логотипа, его размер и относительный путь.
  - Использование в HTML как `src="assets/makosh-logo-mark.png"` и в CSS через класс `.brand-mark`.

### Drift candidates / Кандидаты на drift

Из предоставленного контекста расхождений между кодом, документацией и ADR не выявлено. Все ссылки в HTML ведут на ожидаемые пути в репозитории; CSS‑файл стилизует именно те классы, которые используются в разметке; логотип упоминается в HTML и CSS без разночтений. Бинарный файл не раскрыт, поэтому его содержимое не может быть проверено на соответствие.
