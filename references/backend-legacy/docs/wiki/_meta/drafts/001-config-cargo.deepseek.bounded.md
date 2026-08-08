### Summary / Резюме

Добавить в русскую wiki страницу `operations/configuration.md` документацию по конфигурационному файлу `.cargo/config.toml` — описать его назначение и все четыре псевдонима (`makosh-nextest`, `makosh-nextest-ci`, `makosh-nextest-integration`, `makosh-llvm-cov`), упрощающих запуск тестов и анализ покрытия в проекте `makosh`.

### Proposed pages / Предлагаемые страницы

**`operations/configuration.md`**

```markdown
---
tags: [configuration, cargo, makosh]
---

# Конфигурация Cargo

Файл `.cargo/config.toml` определяет псевдонимы (aliases) для `cargo`, которые стандартизируют запуск тестов через `cargo-nextest` и генерацию LLVM-отчётов о покрытии. Все псевдонимы используют манифест `backend/Cargo.toml`, чтобы явно указывать рабочую область бэкенда.

## Псевдонимы

### `makosh-nextest`

Запускает все тесты проекта с помощью `cargo-nextest`.

```sh
cargo makosh-nextest
```

Раскрывается в команду:

```
nextest run --manifest-path backend/Cargo.toml
```

### `makosh-nextest-ci`

Запускает тесты в профиле `ci`, оптимизированном для конвейеров непрерывной интеграции (меньше лишнего вывода, повторяемые результаты).

```sh
cargo makosh-nextest-ci
```

Раскрывается в команду:

```
nextest run --manifest-path backend/Cargo.toml --profile ci
```

### `makosh-nextest-integration`

Запускает только интеграционные тесты (помеченные `#[cfg(test)]` в интеграционных файлах) с профилем `integration`.

```sh
cargo makosh-nextest-integration
```

Раскрывается в команду:

```
nextest run --manifest-path backend/Cargo.toml --profile integration --tests
```

Флаг `--tests` ограничивает запуск только тестовыми целями, исключая бенчмарки и примеры.

### `makosh-llvm-cov`

Генерирует отчёт о покрытии кода с помощью `cargo-llvm-cov` (инструмент на базе LLVM source-based code coverage).

```sh
cargo makosh-llvm-cov
```

Раскрывается в команду:

```
llvm-cov --manifest-path backend/Cargo.toml
```

## Исходный файл

`.cargo/config.toml` (328 байт)
```

### Source coverage / Покрытие источников

| Исходный файл | Покрытые факты |
|---|---|
| `.cargo/config.toml` | Определение четырёх cargo-алиасов: `makosh-nextest`, `makosh-nextest-ci`, `makosh-nextest-integration`, `makosh-llvm-cov`; их точные раскрытия; путь к манифесту `backend/Cargo.toml`; используемые профили (`ci`, `integration`) и флаг `--tests`. |

### Drift candidates / Кандидаты на drift

Из предоставленного контекста (единственный файл `.cargo/config.toml`) расхождения кода, документации или ADR не видны.
