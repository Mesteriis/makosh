Now I have full context. Let me compose the wiki page.

---

### Summary / Резюме

Создать первую версию страницы `operations/configuration.md` на русском языке для Obsidian wiki. Страница документирует алиасы Cargo из `.cargo/config.toml` — четыре сокращённые команды `cargo makosh-*`, оборачивающие `cargo-nextest` и `cargo-llvm-cov` с указанием манифеста бэкенда. Страница новая, так как директория `operations/` пуста. Другие чанки (002, 004, 005, 007, 014, 016, 019) позже дополнят эту же страницу секциями по `.config/nextest.toml`, `.github/workflows/*`, `.gitignore`, `.pre-commit-config.yaml`, `Cargo.toml`/`Cargo.lock` и `Makefile`.

### Proposed pages / Предлагаемые страницы

#### `operations/configuration.md`

```markdown
# Конфигурация

Конфигурационные файлы и настройки репозитория, влияющие на сборку, тестирование и инструментарий.

## `.cargo/config.toml` — алиасы Cargo

Файл `.cargo/config.toml` определяет сокращённые алиасы для `cargo`, специфичные для проекта Макошь. Все алиасы указывают на манифест бэкенда (`backend/Cargo.toml`) и ускоряют запуск тестов и coverage без необходимости запоминать полные команды.

### Алиасы

- `makosh-nextest` — запуск тестов через `cargo-nextest` с профилем по умолчанию:
  ```toml
  makosh-nextest = "nextest run --manifest-path backend/Cargo.toml"
  ```
  Эквивалент: `cargo nextest run --manifest-path backend/Cargo.toml`. Профиль `default` описан в `.config/nextest.toml`.

- `makosh-nextest-ci` — запуск тестов с CI-профилем (две ретрая, увеличенные таймауты):
  ```toml
  makosh-nextest-ci = "nextest run --manifest-path backend/Cargo.toml --profile ci"
  ```
  Используется в `make test-ci` и GitHub Actions workflow `ci.yml`.

- `makosh-nextest-integration` — запуск интеграционных тестов с профилем `integration` (увеличенный slow-timeout 120s):
  ```toml
  makosh-nextest-integration = "nextest run --manifest-path backend/Cargo.toml --profile integration --tests"
  ```
  Используется в `make test-integration`.

- `makosh-llvm-cov` — запуск `cargo-llvm-cov` для измерения покрытия:
  ```toml
  makosh-llvm-cov = "llvm-cov --manifest-path backend/Cargo.toml"
  ```
  Используется в `make coverage`, `make coverage-html` и `make coverage-ci` через скрипт `scripts/test/run-llvm-cov.sh`.

### Связь с Makefile

Алиасы из `.cargo/config.toml` не вызываются напрямую из Makefile — `make` использует полные команды `cargo nextest` и `cargo llvm-cov`, дублируя флаг `--manifest-path backend/Cargo.toml`. Алиасы предназначены для ручного использования разработчиком в терминале:

```bash
cargo makosh-nextest              # быстрый прогон тестов
cargo makosh-nextest-ci           # прогон с CI-профилем
cargo makosh-nextest-integration  # только интеграционные тесты
cargo makosh-llvm-cov             # покрытие кода
```
```

### Source coverage / Покрытие источников

| Файл | Факты, покрытые на странице |
|---|---|
| `.cargo/config.toml` | Четыре алиаса (`makosh-nextest`, `makosh-nextest-ci`, `makosh-nextest-integration`, `makosh-llvm-cov`), их полные значения, связь с `backend/Cargo.toml`, профили nextest (`default`, `ci`, `integration`), использование в Makefile и при ручной разработке |

### Drift candidates / Кандидаты на drift

Из контекста данного чанка drift между кодом и документацией не виден. Однако отмечу, что Makefile **не использует** алиасы из `.cargo/config.toml` — он дублирует полные команды `cargo nextest` и `cargo llvm-cov`. Это наблюдение, а не drift; Makefile может сознательно не полагаться на локальные алиасы для воспроизводимости CI.
