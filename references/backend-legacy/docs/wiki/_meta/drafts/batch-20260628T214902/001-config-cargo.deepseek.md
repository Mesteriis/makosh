### Summary / Резюме

В русскую Obsidian wiki необходимо добавить страницу `operations/configuration.md`, документирующую псевдонимы Cargo из файла `.cargo/config.toml`. Причина: предоставить инженерам актуальный перечень полезных команд для запуска тестов и анализа покрытия, определённых на уровне репозитория. Текущее содержимое wiki-страницы не встроено в контекст, поэтому предложение является новым разделом.

### Proposed pages / Предлагаемые страницы

- `operations/configuration.md`

```markdown
# Конфигурация

## Псевдонимы Cargo (`.cargo/config.toml`)

Корневой файл `.cargo/config.toml` определяет несколько команд-псевдонимов для `cargo`. Они упрощают запуск тестов и сбор покрытия кода с помощью `cargo nextest` и `cargo llvm-cov`.

| Псевдоним | Фактическая команда |
|---|---|
| `cargo makosh-nextest` | `nextest run --manifest-path backend/Cargo.toml` |
| `cargo makosh-nextest-ci` | `nextest run --manifest-path backend/Cargo.toml --profile ci` |
| `cargo makosh-nextest-integration` | `nextest run --manifest-path backend/Cargo.toml --profile integration --tests` |
| `cargo makosh-llvm-cov` | `llvm-cov --manifest-path backend/Cargo.toml` |

Эти псевдонимы используют `--manifest-path backend/Cargo.toml`, поэтому их необходимо выполнять из корня репозитория.
```

### Source coverage / Покрытие источников

| Файл | Использованные факты |
|---|---|
| `.cargo/config.toml` | Все четыре псевдонима и их точные значения: `makosh-nextest`, `makosh-nextest-ci`, `makosh-nextest-integration`, `makosh-llvm-cov`. |

### Drift candidates / Кандидаты на drift

Нет видимых расхождений в рамках данного чанка, так как существующее содержимое `operations/configuration.md` не встроено в контекст и не может быть сопоставлено с исходным файлом.
