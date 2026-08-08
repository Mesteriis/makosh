---
chunk_id: 135-other-frontend-part-008
batch_id: batch-20260628T214902
group: frontend
role: other
source_status: pending
source_count: 25
generated_by: code-wiki-ru
---

# 135-other-frontend-part-008 — frontend/other

- Target index: [[components/frontend]]
- Batch: `batch-20260628T214902`
- Source files: `25`

## Резюме

Обновить страницу `components/frontend.md` русской Obsidian wiki: добавить обзор архитектуры фронтенд-компонентов проекта `makosh` на основе предоставленных исходных файлов. Описать основные домены (`communications`, `documents`, `home`, `knowledge`, `notes`), их страницы и ключевые компоненты, привести общие паттерны и предупреждения о возможном дрифте.

## Предложенные страницы

### `components/frontend.md`

```markdown
# Компоненты фронтенда (Frontend Components)

## Обзор

Фронтенд `makosh` построен на Vue 3 Composition API с TypeScript. Код организован по доменам (`domains`), каждый из которых содержит собственные представления (`views`), компоненты (`components`), хранилища (`stores`), запросы (`queries`) и типы (`types`).

## Домены и компоненты

### Communications (Коммуникации)

Основная страница: `CommunicationsPage.vue`. В зависимости от выбранной в навигации секции (`nav.activeCommunicationSection`) отображает:

- `TelegramCommunicationsPanel` (код не включён в данный контекст)
- `WhatsAppCommunicationsPanel` (описан ниже)
- `CommunicationsCallsPanel` для режимов `calls` и `meetings`
- Комплексный email-воркбенч: `CommunicationsActionBar`, `CommunicationsListPane`, `CommunicationsDetailPane`, `CommunicationsRailPane`, `ComposeDrawer`, `OutboxStatusStrip`, `SavedSearchStrip`, `CommunicationFolderStrip`, `BulkActionsBar`, `AttachmentSearchPanel` (детали этих компонентов не раскрыты в предоставленных исходниках)

#### WhatsApp панель (`WhatsAppCommunicationsPanel`)

Состоит из трёх файлов:

- **`WhatsAppCommunicationsChatPane.vue`** – основная область чата. Два режима просмотра (`browserMode`): `'timeline'` (сообщения) и `'media'` (медиа).
  - Сообщения содержат: отправитель, время, текст, мета-флаги (системные сообщения, статусы), упоминания, превью ссылок, опросы, геолокации, контактные карточки, стикеры, системные события.
  - Статусные сообщения (`isStatusMessage`) дополнительно показывают автора, просмотры, жизненный цикл, медиа-галерею.
  - Медиа-галерея позволяет превью и переход к исходному сообщению.
  - Реакции: отображаются с возможностью удаления; палитра для добавления использует `TELEGRAM_REACTION_PALETTE` (урезанную до 8 эмодзи).
  - Действия с сообщением: ответ, пересылка, редактирование, удаление.
  - Форма отправки нового сообщения (внизу, обрезана в исходниках, но судя по классу `.provider-inline-form`, содержит поле ввода и кнопку отправки).
- **`WhatsAppCommunicationsDetailPane.vue`** – боковая панель деталей.
  - Свойства беседы: тип (`chat_kind` или `status_feed`), непрочитанность, количество участников, архивирование, заглушение, закрепление.
  - Секция «Edit draft»: редактирование текста сообщения с кнопками сохранения/отмены.
  - Секция «Forward target»: выбор целевой беседы через фильтруемый список.
  - Список участников (до 8), закреплённые сообщения (до 5) с кнопкой перехода, медиа-элементы (до 6) с превью и переходом.
  - «Media preview»: отображение изображений (`kind=image`), аудио, видео, PDF (iframe), текстового представления. Состояния: загрузка, ошибка, усечённое превью, успех.
- **`WhatsAppCommunicationsPanel.vue`** – корневой компонент-оркестратор. Управляет состоянием (локальные `ref`) и связывает панели.
  - Использует множество запросов и мутаций из `whatsappBusinessQueries`: списки бесед, сообщения, поиск сообщений, поиск медиа, закреплённые сообщения, участники; мутации отправки, ответа, пересылки, редактирования, удаления; управление pin/archive/mute/mark read/unread; добавление/удаление реакций.
  - Для безопасного превью вложений вызывает `useAttachmentPreviewQuery`.
- **Стили**: `WhatsAppCommunicationsPanel.css` задаёт раскладку панелей, сообщений, мета-элементов, реакций, медиа-галереи, детальной панели и превью через CSS-классы и переменные (`var(--hh-border)` и т.п.).

#### CommunicationsEmptyPage

`CommunicationsEmptyPage.vue` – компонент-заглушка: иконка, заголовок `communications.empty.title` и описание `communications.empty.description` через `useI18n()`.

### Documents (Документы)

Основная страница: `DocumentsPage.vue`.

- **`DocumentsSourceCards.vue`** – карточки источников (Google Drive, OneDrive, Dropbox, Notion) с фиксированными названиями и количествами.
- **`DocumentsNavigation.vue`** – боковая навигация с жёстко заданными «Smart Collections» и «My Folders».
- **`DocumentsList.vue`** – виртуализированный (TanStack Virtual) список документов. Фильтр (All / Shared / Recent), локальный поиск. Отображает иконку, имя, источник, проект, тип, размер, дату.
- **`DocumentsProcessingJobs.vue`** – список задач обработки. Показывает `document_id`, статус (`status`), шаг, время постановки в очередь. Для задач со статусом `failed` – кнопка повтора.
- **`DocumentsInsights.vue`** – плейсхолдер: «AI analysis results will appear here when document processing is complete.»

Страница использует `useDocumentsStore` (Pinia) для состояния поиска, фильтра, идентификатора повторяемой задачи и строки ошибки. Запросы – через `useDocumentProcessingJobsQuery`.

### Home (Главная)

Основная страница: `HomePage.vue`.

- **`HomeMetrics.vue`** – сетка из 6 метрик, включая «Focus Score» с кольцевым индикатором. Данные метрик частично строятся из `mailboxHealth` (total_messages, needs_action, waiting, и т.д.).
- **`HomeWhatsNew.vue`** – лента «What's New», наполняется из последних сообщений (`useCommunicationMessagesQuery`).
- **`HomePriorities.vue`** – «Today's Priorities»: список задач с чекбоксами. В текущих исходниках массив задач пуст (`[] as TaskItem[]`).
- **`HomeUpcoming.vue`** – «Upcoming»: события на сегодня и завтра (жёстко заданы).
- **`HomePeopleTalked.vue`** – «People You Talked To»: список уникальных отправителей из последних сообщений.
- **`HomeSystemStatus.vue`** – «System Status»: статический список статусов, плюс поле `statusError` (пустое).
- **`HomeActiveProjects.vue`** – «Active Projects»: проекты с прогресс-барами (передаётся пустой массив), кнопка перехода на страницу проектов.

### Knowledge (Знания)

Основная страница: `KnowledgePage.vue`.

- Фильтры-чипсы (`graphFilterChips`) по типам узлов; кнопка «Rebuild».
- Форма поиска узлов графа; результаты поиска.
- **`KnowledgeGraphCanvas.vue`** – SVG-холст графа с узлами и рёбрами. Состояния: загрузка, ошибка, пустой граф, ожидание выбора.
- Боковая панель:
  - **`KnowledgePolygraphReview.vue`** – обзор противоречий (`ContradictionObservation`) с серьёзностью (critical/high/medium/low), карточками утверждений, источниками, кнопками подтверждения/отклонения.
  - **`KnowledgeNodeInspector.vue`** – инспектор узла: тип, метка, свойства, количество связей по типам, evidence, общая статистика графа.

Использует `useKnowledgeStore`. Запросы: `useGraphSummaryQuery`, `useContradictionsQuery`.

### Notes (Заметки)

- **`NotesInsights.vue`** – плейсхолдер: «AI generated summaries and connections across your notes will appear here.»

## Общие паттерны

- **Интернационализация**: `useI18n()` из `platform/i18n` для строк интерфейса.
- **Иконки**: компонент `Icon` с атрибутом `icon` (строка), `width`, `height` или `size`.
- **Композиция**: страницы собираются из пропсо-событийных компонентов.
- **Состояние**: Pinia-хранилища (`useNavigationStore`, `useDocumentsStore`, `useKnowledgeStore`) и локальные `ref`/`computed`.
- **Виртуализация**: TanStack Virtual (`@tanstack/vue-virtual`) в списке документов.
- **Запросы**: вынесены в модули `queries`, предположительно TanStack Query.
- **Стилизация**: CSS-переменные дизайн-системы (`--hh-border`, `--hh-bg-primary`, `--hh-color-accent` и др.).

## Дополнительные замечания

- В WhatsApp-компонентах используется `TELEGRAM_REACTION_PALETTE` – кросс-провайдерное переиспользование.
- Вспомогательные функции для WhatsApp вынесены в `WhatsAppCommunicationsPanel.helpers` (не включён в контекст).
- Домашняя страница связывает домены Home и Communications через `useCommunicationMessagesQuery`.
- В `CommunicationsPage` условный рендеринг: при активной секции `whatsapp` монтируется `WhatsAppCommunicationsPanel`, при `telegram` – `TelegramCommunicationsPanel`, иначе – полноценный email-воркбенч.
- Email-воркбенч включает множество компонентов, детали которых не подтверждены текущим контекстом.
```

## Покрытие источников

- `frontend/src/domains/communications/providers/whatsapp/views/WhatsAppCommunicationsChatPane.vue` – режимы timeline/media, отображение сообщений (мета-флаги, упоминания, статусы, превью ссылок, опросы, геолокации, контакты, стикеры, системные события), статусная медиа-галерея, реакции с палитрой, действия с сообщением.
- `frontend/src/domains/communications/providers/whatsapp/views/WhatsAppCommunicationsDetailPane.vue` – свойства беседы, секции редактирования черновика и цели пересылки, участники, закреплённые сообщения, медиа-элементы, медиа-превью (виды, состояния).
- `frontend/src/domains/communications/providers/whatsapp/views/WhatsAppCommunicationsPanel.css` – CSS-классы раскладки панелей, сообщений, мета-элементов, реакций, медиа, превью.
- `frontend/src/domains/communications/providers/whatsapp/views/WhatsAppCommunicationsPanel.vue` – оркестрация состояния, запросы/мутации из `whatsappBusinessQueries`, `useAttachmentPreviewQuery`, вспомогательные функции, флаги беседы.
- `frontend/src/domains/communications/views/CommunicationsEmptyPage.vue` – пустое состояние с иконкой и i18n-строками.
- `frontend/src/domains/communications/views/CommunicationsPage.vue` – условный рендеринг провайдерских панелей и email-воркбенча.
- `frontend/src/domains/documents/components/DocumentsInsights.vue` – плейсхолдер AI-инсайтов.
- `frontend/src/domains/documents/components/DocumentsList.vue` – виртуализированный список с фильтрами и поиском.
- `frontend/src/domains/documents/components/DocumentsNavigation.vue` – навигация с коллекциями и папками.
- `frontend/src/domains/documents/components/DocumentsProcessingJobs.vue` – список задач обработки с кнопкой повтора.
- `frontend/src/domains/documents/components/DocumentsSourceCards.vue` – карточки источников.
- `frontend/src/domains/documents/views/DocumentsPage.vue` – композиция страницы документов, использование store и запросов.
- `frontend/src/domains/home/components/HomeActiveProjects.vue` – виджет активных проектов.
- `frontend/src/domains/home/components/HomeMetrics.vue` – виджет метрик и Focus Score.
- `frontend/src/domains/home/components/HomePeopleTalked.vue` – виджет людей.
- `frontend/src/domains/home/components/HomePriorities.vue` – виджет приоритетов (пустые данные).
- `frontend/src/domains/home/components/HomeSystemStatus.vue` – виджет статуса системы.
- `frontend/src/domains/home/components/HomeUpcoming.vue` – виджет предстоящих событий.
- `frontend/src/domains/home/components/HomeWhatsNew.vue` – виджет ленты новостей.
- `frontend/src/domains/home/views/HomePage.vue` – композиция домашней страницы, связь с запросами сообщений и здоровья ящика.
- `frontend/src/domains/knowledge/components/KnowledgeGraphCanvas.vue` – SVG-граф с состояниями.
- `frontend/src/domains/knowledge/components/KnowledgeNodeInspector.vue` – инспектор узла (свойства, связи, evidence, статистика).
- `frontend/src/domains/knowledge/components/KnowledgePolygraphReview.vue` – обзор противоречий с действиями.
- `frontend/src/domains/knowledge/views/KnowledgePage.vue` – композиция страницы знаний (фильтры, поиск, граф, боковая панель).
- `frontend/src/domains/notes/components/NotesInsights.vue` – плейсхолдер инсайтов заметок.

## Исходные файлы

- [`frontend/src/domains/communications/providers/whatsapp/views/WhatsAppCommunicationsChatPane.vue`](../../../../frontend/src/domains/communications/providers/whatsapp/views/WhatsAppCommunicationsChatPane.vue)
- [`frontend/src/domains/communications/providers/whatsapp/views/WhatsAppCommunicationsDetailPane.vue`](../../../../frontend/src/domains/communications/providers/whatsapp/views/WhatsAppCommunicationsDetailPane.vue)
- [`frontend/src/domains/communications/providers/whatsapp/views/WhatsAppCommunicationsPanel.css`](../../../../frontend/src/domains/communications/providers/whatsapp/views/WhatsAppCommunicationsPanel.css)
- [`frontend/src/domains/communications/providers/whatsapp/views/WhatsAppCommunicationsPanel.vue`](../../../../frontend/src/domains/communications/providers/whatsapp/views/WhatsAppCommunicationsPanel.vue)
- [`frontend/src/domains/communications/views/CommunicationsEmptyPage.vue`](../../../../frontend/src/domains/communications/views/CommunicationsEmptyPage.vue)
- [`frontend/src/domains/communications/views/CommunicationsPage.vue`](../../../../frontend/src/domains/communications/views/CommunicationsPage.vue)
- [`frontend/src/domains/documents/components/DocumentsInsights.vue`](../../../../frontend/src/domains/documents/components/DocumentsInsights.vue)
- [`frontend/src/domains/documents/components/DocumentsList.vue`](../../../../frontend/src/domains/documents/components/DocumentsList.vue)
- [`frontend/src/domains/documents/components/DocumentsNavigation.vue`](../../../../frontend/src/domains/documents/components/DocumentsNavigation.vue)
- [`frontend/src/domains/documents/components/DocumentsProcessingJobs.vue`](../../../../frontend/src/domains/documents/components/DocumentsProcessingJobs.vue)
- [`frontend/src/domains/documents/components/DocumentsSourceCards.vue`](../../../../frontend/src/domains/documents/components/DocumentsSourceCards.vue)
- [`frontend/src/domains/documents/views/DocumentsPage.vue`](../../../../frontend/src/domains/documents/views/DocumentsPage.vue)
- [`frontend/src/domains/home/components/HomeActiveProjects.vue`](../../../../frontend/src/domains/home/components/HomeActiveProjects.vue)
- [`frontend/src/domains/home/components/HomeMetrics.vue`](../../../../frontend/src/domains/home/components/HomeMetrics.vue)
- [`frontend/src/domains/home/components/HomePeopleTalked.vue`](../../../../frontend/src/domains/home/components/HomePeopleTalked.vue)
- [`frontend/src/domains/home/components/HomePriorities.vue`](../../../../frontend/src/domains/home/components/HomePriorities.vue)
- [`frontend/src/domains/home/components/HomeSystemStatus.vue`](../../../../frontend/src/domains/home/components/HomeSystemStatus.vue)
- [`frontend/src/domains/home/components/HomeUpcoming.vue`](../../../../frontend/src/domains/home/components/HomeUpcoming.vue)
- [`frontend/src/domains/home/components/HomeWhatsNew.vue`](../../../../frontend/src/domains/home/components/HomeWhatsNew.vue)
- [`frontend/src/domains/home/views/HomePage.vue`](../../../../frontend/src/domains/home/views/HomePage.vue)
- [`frontend/src/domains/knowledge/components/KnowledgeGraphCanvas.vue`](../../../../frontend/src/domains/knowledge/components/KnowledgeGraphCanvas.vue)
- [`frontend/src/domains/knowledge/components/KnowledgeNodeInspector.vue`](../../../../frontend/src/domains/knowledge/components/KnowledgeNodeInspector.vue)
- [`frontend/src/domains/knowledge/components/KnowledgePolygraphReview.vue`](../../../../frontend/src/domains/knowledge/components/KnowledgePolygraphReview.vue)
- [`frontend/src/domains/knowledge/views/KnowledgePage.vue`](../../../../frontend/src/domains/knowledge/views/KnowledgePage.vue)
- [`frontend/src/domains/notes/components/NotesInsights.vue`](../../../../frontend/src/domains/notes/components/NotesInsights.vue)

## Кандидаты на drift

- В `WhatsAppCommunicationsChatPane.vue` используется `TELEGRAM_REACTION_PALETTE` (типы telegram) в контексте WhatsApp. Возможно, это намеренное переиспользование, но может указывать на дрифт — ожидание отдельной палитры для WhatsApp.
- `HomePriorities` получает пустой массив задач (`[] as TaskItem[]`), что выглядит как нереализованная интеграция с реальными данными.
- В `DocumentsPage` список документов (`documents`) целиком строится из `processingJobs`, что может быть временным маппингом до появления полноценного списка документов.
- Многие email-компоненты в `CommunicationsPage` (ActionBar, ListPane, DetailPane, и т.д.) упомянуты, но их исходный код не включён в контекст; утверждения о них в wiki не подтверждены.
- `WhatsAppCommunicationsPanel.css` содержит классы `.message-bubble--flash`, которые не используются в предоставленных шаблонах чата — возможный дрифт между стилями и разметкой.
