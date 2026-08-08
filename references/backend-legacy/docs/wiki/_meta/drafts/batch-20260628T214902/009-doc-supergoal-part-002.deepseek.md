### Summary / Резюме

Страница `operations/documentation-map.md` должна быть создана или обновлена, чтобы описать структуру и содержание операционной документации, связанной с проектом миграции фронтенда (supergoal `makosh-frontend-migration-vue-3-WzENWm`). В текущем контексте доступны фазовые планы (2–15), карта репозитория (`repo-map.md`) и перечень инструментов (`tools.md`). Страница документирует состав плана, его фазы и вспомогательные файлы на русском языке.

### Proposed pages / Предлагаемые страницы

#### `operations/documentation-map.md`

```markdown
# Карта документации

В этом разделе собрана операционная документация, относящаяся к проекту миграции фронтенда Макошь с SvelteKit на Vue 3 (`supergoal`).

## План миграции (supergoal)

**Supergoal**: `makosh-frontend-migration-vue-3-WzENWm`
Расположение: `../../.supergoal/makosh-frontend-migration-vue-3-WzENWm/`

План состоит из 15 фаз. Каждая фаза описана в отдельном `phase-N.md` файле. Ниже представлены фазы, встроенные в данный контекст:

| Фаза | Название (Task) | Краткое описание | Зависимости от фаз |
|------|------------------|------------------|-------------------|
| 2 | App Shell | Перенос Sidebar, Topbar, workspace layout, notifications drawer, layout editor; настройка Vue Router | 1 |
| 3 | Shared UI Primitives | Инициализация shadcn-vue, создание общих UI-компонентов (Button, Input, Dialog и др.) | 1 |
| 4 | Settings Domain | Перенос страницы настроек со всеми панелями | 1,2,3 |
| 5 | Home Dashboard | Перенос домашней панели с виджетами | 1,2,3 |
| 6 | Personas & Organizations | Перенос доменов Персон и Организаций (list + detail, identity review) | 1,2,3 |
| 7 | Projects & Tasks | Перенос доменов Проектов и Задач (с TanStack Virtual) | 1,2,3 |
| 8 | Calendar Domain | Перенос календаря с событиями | 1,2,3 |
| 9 | Documents & Notes | Перенос документов и заметок (списки с TanStack Virtual, фильтры, insights) | 1,2,3 |
| 10 | Knowledge & Review | Перенос графа знаний (Vue Flow) и обзора противоречий (Polygraph) | 1,2,3 |
| 11 | Agents & Timeline | Перенос агентов (статус AI) и ленты событий (TanStack Virtual) | 1,2,3 |
| 12 | Communications/Mail | Перенос почтового домена — самого сложного (Mail list, Message viewer, Compose drawer и др.) | 1,2,3 |
| 13 | Telegram & WhatsApp | Перенос Telegram и WhatsApp (подсекции Communications) | 1,2,3 |
| 14 | Polish & Harden | Полировка: UX-тексты, состояния (loading/empty/error/unauthorized), граничные случаи, безопасность, доступность (a11y), производительность, анимации, регрессионный обзор | 1–13 |
| 15 | Cutover | Финальное удаление SvelteKit-кода, обновление `tauri.conf.json`, Makefile, CI/CD, полный валидационный прогон | 14 |

> **Примечание**: Фаза 1 (Setup) не встроена в данный контекст, поэтому её содержание не подтверждено.

### Структура фазового файла

Каждый файл фазы содержит:
- **Заголовок** `SUPERGOAL_PHASE_START` с номером фазы, общим числом фаз и названием.
- **Task** — краткая формулировка задачи.
- **Mandatory commands** — обязательные команды (например, `cd frontend && pnpm build`).
- **Acceptance criteria (AC)** — критерии приёмки (от 4 до 10 пунктов).
- **Depends on phases** — перечень фаз, от которых зависит данная.
- **Work** — пошаговое описание работ.
- **Notes** — примечания, включая ссылки на Svelte-реализации в `frontend/src-svelte/lib/pages/...` и рекомендации по архитектуре (Pinia для UI-состояния, TanStack Query для серверного, лимит 500 строк на компонент).

## Вспомогательные документы плана

### `repo-map.md` (карта репозитория)

Сгенерирован **2026-06-14 13:26:32**. Содержит:

- **Верхнеуровневую структуру**: корневые каталоги (`backend`, `frontend`, `crates`, `docs`, `scripts`, `docker` и др.), а также ключевые файлы (`AGENTS.md`, `README.md`, `IMPLEMENTATION_STATUS.md`).
- **Распределение файлов по расширениям**: `.rs` — 1024, `.md` — 229, `.ts` — 164, `.svelte` — 121, `.sql` — 74 и другие.
- **Крупнейшие файлы**: лидируют backend-тесты (`email_account_setup.rs`, `telegram.rs`, `calendar_api.rs` и т.д.).
- **Тестовая поверхность**: 201 директория `tests`, 4 директории `test`, 40 тестовых файлов по имени.
- **Конфигурация CI/CD**: `.github/workflows`.
- **Последние коммиты**: рефакторинг границ (`split ... boundary`) в backend, датированный 2026-06-13 – 2026-06-14.

### `tools.md` (доступные инструменты)

Описывает инструменты, доступные на **Stage 0** для работы над supergoal:

- **Доступны**: `codebase_search`, `search_files`, `list_files`, `read_file`, `execute_command`, `write_to_file`, `apply_diff`, `ask_followup_question`, `new_task` (субагент), `skill`.
- **Недоступны**: `WebSearch`, `WebFetch`, `Context7`, `MCP clients`.

Это означает, что при планировании и выполнении фаз полагаться можно только на локальное исследование кода, чтение файлов и знания модели о Vue 3, TanStack, Tailwind и shadcn-vue.
```

### Source coverage / Покрытие источников

| Источник | Факты, покрытые в предлагаемой странице |
|----------|----------------------------------------|
| `.supergoal/makosh-frontend-migration-vue-3-WzENWm/phases/phase-2.md` | Фаза 2: задача, зависимости, критерии приёмки (10), обязательная команда `cd frontend && pnpm build` |
| `.supergoal/makosh-frontend-migration-vue-3-WzENWm/phases/phase-3.md` | Фаза 3: задача, зависимости, критерии приёмки (8) |
| `.supergoal/makosh-frontend-migration-vue-3-WzENWm/phases/phase-4.md` | Фаза 4: задача, зависимости, критерии приёмки (7) |
| `.supergoal/makosh-frontend-migration-vue-3-WzENWm/phases/phase-5.md` | Фаза 5: задача, зависимости, критерии приёмки (5) |
| `.supergoal/makosh-frontend-migration-vue-3-WzENWm/phases/phase-6.md` | Фаза 6: задача, зависимости, критерии приёмки (6) |
| `.supergoal/makosh-frontend-migration-vue-3-WzENWm/phases/phase-7.md` | Фаза 7: задача, зависимости, критерии приёмки (5) |
| `.supergoal/makosh-frontend-migration-vue-3-WzENWm/phases/phase-8.md` | Фаза 8: задача, зависимости, критерии приёмки (4) |
| `.supergoal/makosh-frontend-migration-vue-3-WzENWm/phases/phase-9.md` | Фаза 9: задача, зависимости, критерии приёмки (5) |
| `.supergoal/makosh-frontend-migration-vue-3-WzENWm/phases/phase-10.md` | Фаза 10: задача, зависимости, критерии приёмки (6) |
| `.supergoal/makosh-frontend-migration-vue-3-WzENWm/phases/phase-11.md` | Фаза 11: задача, зависимости, критерии приёмки (5) |
| `.supergoal/makosh-frontend-migration-vue-3-WzENWm/phases/phase-12.md` | Фаза 12: задача, зависимости, критерии приёмки (9), описание как самого сложного домена |
| `.supergoal/makosh-frontend-migration-vue-3-WzENWm/phases/phase-13.md` | Фаза 13: задача, зависимости, критерии приёмки (7) |
| `.supergoal/makosh-frontend-migration-vue-3-WzENWm/phases/phase-14.md` | Фаза 14: задача, зависимости, критерии приёмки (9), 9 подпроходов (UX, States, Edges, Security, A11y, Performance, Diff Review, Regression Sweep, Animation) |
| `.supergoal/makosh-frontend-migration-vue-3-WzENWm/phases/phase-15.md` | Фаза 15: задача, зависимости, критерии приёмки (8), шаги (удаление SvelteKit, обновление конфигов, валидационный прогон) |
| `.supergoal/makosh-frontend-migration-vue-3-WzENWm/repo-map.md` | Карта репозитория: структура, статистика файлов, тесты, CI, недавняя активность |
| `.supergoal/makosh-frontend-migration-vue-3-WzENWm/tools.md` | Перечень доступных/недоступных инструментов на Stage 0 |

### Drift candidates / Кандидаты на drift

- **Отсутствие фазы 1**: Файл `phase-1.md` не встроен в контекст, хотя фаза 2 и последующие указывают его как зависимость. Содержание фазы 1 не может быть подтверждено из предоставленных источников.
- **Планы против реального кода**: Все фазовые файлы являются планами миграции и содержат указания на компоненты и структуры, которые ещё предстоит создать. Соответствие этих планов фактическому состоянию репозитория не может быть проверено в рамках данного контекста.
- **Ссылки на Svelte-исходники**: Фазы содержат прямые ссылки на `frontend/src-svelte/lib/...`, но содержимое этих директорий не встроено. Их актуальность и полнота не подтверждены.
- **`IMPLEMENTATION_STATUS.md`**: Упомянут в `repo-map.md` как активно изменяемый файл (20 коммитов), однако его содержимое не предоставлено, и связь между ним и планом миграции неизвестна.
