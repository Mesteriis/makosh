---
chunk_id: 007-config-pre-commit-config
batch_id: batch-20260628T214902
group: .pre-commit-config
role: config
source_status: pending
source_count: 1
generated_by: code-wiki-ru
---

# 007-config-pre-commit-config — .pre-commit-config/config

- Target index: [[operations/configuration]]
- Batch: `batch-20260628T214902`
- Source files: `1`

## Резюме

Добавить в страницу `operations/configuration.md` описание локальных pre‑commit‑хуков, определённых в `.pre-commit-config.yaml`.
Текущий файл задаёт пять проверок, каждая из которых запускает конкретную Make‑цель, и фильтрует срабатывание по изменённым файлам.
Без этой документации контрибьюторы и сопровождающие не имеют явного перечня автоматических проверок, что затрудняет настройку окружения, отладку и понимание ожидаемого поведения CI.

## Предложенные страницы

#### `operations/configuration.md`

```markdown
# Конфигурация

## `.pre-commit-config.yaml`

Файл `.pre-commit-config.yaml` в корне репозитория содержит локальные (`repo: local`) хуки предкоммитных проверок.

Каждый хук использует `language: system` и запускается без передачи имён файлов (`pass_filenames: false`).

| ID                        | Название                                       | Команда                      | Условие срабатывания                                                                                                                                                                                                 |
|---------------------------|------------------------------------------------|------------------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `makosh-architecture-guards` | Макошь architecture and code guards            | `make architecture-check`    | Всегда (`always_run: true`)                                                                                                                                                                                          |
| `makosh-code-boundaries`    | Макошь code boundary guards                    | `make code-boundaries-check` | При изменениях в `backend/src/`, `frontend/src/` или `Makefile` (`files: ^(backend/src/\|frontend/src/\|Makefile$)`)                                                                                               |
| `makosh-rust-fmt`           | Макошь Rust format check                        | `make backend-fmt-check`     | При изменениях в файлах с расширением `.rs` внутри `backend/` (`files: ^backend/.*\\.rs$`)                                                                                                                            |
| `makosh-rust-clippy`       | Макошь Rust clippy                              | `make backend-clippy`        | При изменениях в файлах с расширением `.rs` внутри `backend/` (`files: ^backend/.*\\.rs$`)                                                                                                                            |
| `makosh-frontend-lint`     | Макошь frontend lint (style + TypeScript/Vue, no tests) | `make frontend-lint`         | При изменениях в `frontend/(src\|static\|scripts)/` или в `frontend/package.json`, `frontend/pnpm-lock.yaml`, `frontend/tsconfig.json`, `frontend/vite.config.ts` (`files: ^frontend/(src\|static\|scripts)/\|^frontend/(package\\.json\|pnpm-lock\\.yaml\|tsconfig\\.json\|vite\\.config\\.ts)$`) |

Конкретный смысл выполняемых Make‑команд (например, какие именно инструменты вызываются внутри `make architecture-check`) не подтверждён данным контекстом и здесь не описан.
```

## Покрытие источников

**`.pre-commit-config.yaml`**

- В файле определён один блок `repo: local` без удалённых репозиториев.
- Перечислены пять хуков с идентификаторами: `makosh-architecture-guards`, `makosh-code-boundaries`, `makosh-rust-fmt`, `makosh-rust-clippy`, `makosh-frontend-lint`.
- Для каждого хука заданы `entry` (команда Make), `language: system`, `pass_filenames: false`.
- Хук `makosh-architecture-guards` имеет `always_run: true` и не содержит фильтр `files`.
- Остальные четыре хука содержат поле `files` с регулярными выражениями для пути/имени файла.
- Все команды `entry` являются Make‑целями: `make architecture-check`, `make code-boundaries-check`, `make backend-fmt-check`, `make backend-clippy`, `make frontend-lint`.
- Файл является валидным YAML; на данном уровне нет указаний на условные профили, переменные окружения или вызовы внешних инструментов, кроме Make.

## Исходные файлы

- [`.pre-commit-config.yaml`](../../../../.pre-commit-config.yaml)

## Кандидаты на drift

На основании предоставленного контекста расхождений между кодом, документацией или архитектурными решениями не выявлено.
Другие файлы (включая сам Makefile, скрипты проверок, конфигурации линтеров/форматтеров и содержимое `docs/wiki/`) не были включены в данный context pack, поэтому подтвердить или опровергнуть дрейф невозможно.
