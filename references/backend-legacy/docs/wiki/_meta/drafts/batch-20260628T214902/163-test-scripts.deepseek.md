### Summary / Резюме

Добавить страницу `operations/scripts-tests.md`, описывающую скрипты автоматизации тестов из `scripts/test/`: `analyze-nextest-junit.mjs`, `backend-test-targets.mjs`, `collect-performance-reports.sh`, `run-nextest.sh` и `run-llvm-cov.sh`. Страница на русском, совместимая с Obsidian Markdown, содержит только факты, прямо подтверждённые встроенными исходниками.

### Proposed pages / Предлагаемые страницы

#### operations/scripts-tests.md

```markdown
# Test-скрипты

Скрипты в `scripts/test/` автоматизируют запуск тестов, сбор отчётов о производительности и покрытии, а также анализ JUnit-вывода `cargo-nextest`.

## `analyze-nextest-junit.mjs`

- **Назначение**: парсит JUnit XML (генерируемый `cargo nextest`) и формирует сводный отчёт в JSON и Markdown.
- **Запуск**: `node scripts/test/analyze-nextest-junit.mjs --input <junit.xml> --output <path-prefix> [--suite <name>]`
- **Что делает**:
  - Извлекает все `<testcase>` из XML, собирает: `classname`, `name`, `time` (сек), наличие `<failure>` или `<error>` (failed), наличие `<flakyFailure>` (flaky).
  - Сортирует результаты по времени, вычисляет общую длительность, среднюю, p95, p99.
  - Определяет 10 самых медленных тестов.
  - Генерирует JSON-отчёт (`<output>.json`) и Markdown-отчёт (`<output>.md`).
  - Выводит в консоль строку прогресса (progress bar) и путь к отчёту.
- **Выходные файлы**:
  - `<output>.json` — машинночитаемая статистика.
  - `<output>.md` — отчёт с заголовком, сводкой, списком самых медленных тестов и пометками (flaky, failed).
- **Вспомогательные функции** (не экспортируются):
  - `parseArgs` — разбор аргументов `--key value`.
  - `parseAttributes` — разбор XML-атрибутов.
  - `percentile` — вычисление перцентиля из отсортированного массива.
  - `progressBar` — текстовая шкала прогресса.

## `backend-test-targets.mjs`

- **Назначение**: выводит список целей интеграционных тестов (`backend/tests/*.rs`) с категоризацией и подсчётом количества `#[test]`/`#[tokio::test]` в кодовой базе.
- **Запуск**:
  - Сводка (режим по умолчанию): `node scripts/test/backend-test-targets.mjs`
  - Список целей для конкретной категории: `node scripts/test/backend-test-targets.mjs targets <category>`
    - `<category>` одно из: `architecture`, `snapshot`, `e2e`, `integration`.
- **Что делает**:
  - Считывает имена файлов (без `.rs`) из `backend/tests/`.
  - Категоризирует по правилам:
    - `architecture` — имя содержит `architecture`.
    - `snapshot` — имя содержит `snapshot`.
    - `e2e` — имя заканчивается на `_api`, `_stream_api`, `_websocket_api`, `_long_poll_api`, содержит `connectrpc`, равно `hard_v1_routes` или `omniroute`.
    - `integration` — всё остальное.
  - Рекурсивно обходит `backend/src` и `crates/testkit/src` (функция `BunLikeWalk`) и подсчитывает вхождения `#[test]` и `#[tokio::test]` (через regex `/#\[(?:tokio::)?test\]/g`).
  - В режиме `targets` печатает имена целей через пробел.
  - В режиме `summary` выводит JSON: временная метка, `rustUnitTestAttributes` (общее количество юнит-тестов), `backendIntegrationTargets` (количество целей), `categories` (объект с каждой категорией: количество и список целей).

## `collect-performance-reports.sh`

- **Назначение**: собирает отчёты о производительности nextest из JUnit-файлов, найденных в `target/nextest/`.
- **Запуск**: `bash scripts/test/collect-performance-reports.sh`
- **Что делает**:
  - Определяет соответствие входных файлов и имён suite:
    - `target/nextest/default/junit.xml` → suite `default`
    - `target/nextest/ci/junit.xml` → suite `ci`
    - `target/nextest/integration/junit.xml` → suite `integration`
  - Для каждого существующего файла вызывает `scripts/test/analyze-nextest-junit.mjs` с `--input`, `--suite` и `--output reports/test-performance/<suite>`.
  - Если не найдено ни одного JUnit-файла, выводит ошибку и предложение запустить `make test-unit` (или аналогичную nextest-команду).
- **Важно**: скрипт ожидает файлы относительно корня репозитория. Если `cargo nextest` был запущен с `CARGO_TARGET_DIR`, отличным от дефолтного, пути могут не совпадать.

## `run-nextest.sh`

- **Назначение**: запускает `cargo nextest run` с заданным профилем через вспомогательный бинарный файл `makosh_test_session`.
- **Запуск**: `bash scripts/test/run-nextest.sh [profile] [-- дополнительные аргументы nextest]`
  - `profile` — имя nextest-профиля, по умолчанию `default`.
- **Что делает**:
  - Подключает библиотеку `scripts/lib/rust-tooling.sh` (содержит `require_cargo_subcommand`).
  - Проверяет наличие `cargo nextest`.
  - Устанавливает переменные окружения:
    - `CARGO_TARGET_DIR` — по умолчанию `target/validate-test` (если не задана).
    - `CARGO_INCREMENTAL` = `0` (по умолчанию).
    - `NEXTEST_SHOW_PROGRESS` — по умолчанию `bar`.
  - Запускает: `cargo run --manifest-path crates/testkit/Cargo.toml --bin makosh_test_session -- cargo nextest run --manifest-path backend/Cargo.toml --profile ${PROFILE} --show-progress ${NEXTEST_SHOW_PROGRESS} --test-threads ${MAKOSH_NEXTEST_JOBS:-4} "$@"`.
  - Дополнительные аргументы `$@` передаются напрямую в `cargo nextest`.
- **Переменные окружения**:
  - `MAKOSH_NEXTEST_JOBS` — количество потоков тестирования (по умолчанию `4`).
  - `NEXTEST_SHOW_PROGRESS` — формат индикатора прогресса (по умолчанию `bar`).
  - `CARGO_TARGET_DIR` — путь к целевой директории сборки тестов (по умолчанию `target/validate-test`).
  - `CARGO_INCREMENTAL` — отключение инкрементальной компиляции (по умолчанию `0`).

## `run-llvm-cov.sh`

- **Назначение**: запускает `cargo llvm-cov nextest` для сбора покрытия кода вместе с прогоном тестов.
- **Запуск**: `bash scripts/test/run-llvm-cov.sh [profile] [-- дополнительные аргументы]`
  - `profile` — имя профиля, по умолчанию `ci`.
- **Что делает**:
  - Подключает `scripts/lib/rust-tooling.sh`.
  - Проверяет наличие `cargo llvm-cov` и `cargo nextest`.
  - Устанавливает:
    - `CARGO_TARGET_DIR` — по умолчанию `target/coverage-build`.
    - `CARGO_INCREMENTAL` = `0`.
  - Выполняет очистку предыдущих данных покрытия: `cargo llvm-cov clean --workspace`.
  - Запускает: `cargo run --manifest-path crates/testkit/Cargo.toml --bin makosh_test_session -- cargo llvm-cov nextest --manifest-path backend/Cargo.toml --profile ${PROFILE} --show-progress ${NEXTEST_SHOW_PROGRESS} --test-threads ${MAKOSH_NEXTEST_JOBS:-4} "$@"`.
- **Переменные окружения**:
  - `CARGO_TARGET_DIR` — путь для артефактов покрытия (по умолчанию `target/coverage-build`).
  - `NEXTEST_SHOW_PROGRESS`, `MAKOSH_NEXTEST_JOBS`, `CARGO_INCREMENTAL` — аналогично `run-nextest.sh`.
```

### Source coverage / Покрытие источников

- **`scripts/test/analyze-nextest-junit.mjs`** — то, как скрипт парсит аргументы, извлекает атрибуты тестов, вычисляет перцентили, формирует JSON и Markdown отчёты, выводит прогресс-бар, а также форматы выходных файлов.
- **`scripts/test/backend-test-targets.mjs`** — источник правил категоризации целей тестов (`architecture`, `snapshot`, `e2e`, `integration`), способ подсчёта `#[test]`/`#[tokio::test]`, два режима работы (`summary` и `targets`), структура выводимого JSON.
- **`scripts/test/collect-performance-reports.sh`** — маппинг входных JUnit-файлов (`target/nextest/default/junit.xml`, `ci`, `integration`) на имена suite, вызов `analyze-nextest-junit.mjs`, путь для выходных отчётов (`reports/test-performance/<suite>`), поведение при отсутствии файлов.
- **`scripts/test/run-nextest.sh`** — принимаемый профиль (по умолчанию `default`), переменные окружения (`CARGO_TARGET_DIR`, `CARGO_INCREMENTAL`, `NEXTEST_SHOW_PROGRESS`, `MAKOSH_NEXTEST_JOBS`), использование `makosh_test_session`, передача дополнительных аргументов в `cargo nextest`.
- **`scripts/test/run-llvm-cov.sh`** — принимаемый профиль (по умолчанию `ci`), использование `cargo llvm-cov nextest`, очистка `cargo llvm-cov clean --workspace`, переменные окружения (аналогичные `run-nextest.sh`, но `CARGO_TARGET_DIR` по умолчанию `target/coverage-build`), запуск через `makosh_test_session`.

### Drift candidates / Кандидаты на drift

- Несоответствие целевых директорий сборки: `collect-performance-reports.sh` ожидает JUnit-файлы по фиксированным путям `target/nextest/{default,ci,integration}/junit.xml`. В то же время `run-nextest.sh` по умолчанию использует `CARGO_TARGET_DIR=target/validate-test`, а `run-llvm-cov.sh` — `target/coverage-build`. Если эти переменные окружения не переопределены перед запуском `collect-performance-reports.sh`, отчёты могут не найтись. Контекст не содержит явного механизма синхронизации этих путей.
- Внешняя команда `make test-unit`, предлагаемая в сообщении об ошибке `collect-performance-reports.sh`, не встроена в этот context pack; её фактическое поведение и соответствие использованным путям не подтверждено.
