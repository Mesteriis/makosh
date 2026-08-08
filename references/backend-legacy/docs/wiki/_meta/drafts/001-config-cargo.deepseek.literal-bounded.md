### Summary / Резюме

Добавить на страницу `operations/configuration.md` описание файла `.cargo/config.toml` и определённых в нём псевдонимов Cargo. Страница ранее отсутствовала, поэтому создаётся с нуля на основе предоставленного контекста.

### Proposed pages / Предлагаемые страницы

#### operations/configuration.md

```markdown
# Конфигурация

## `.cargo/config.toml`

Файл `.cargo/config.toml` задаёт псевдонимы (aliases) для запуска инструментов сборки и тестирования.

### Псевдонимы

| Псевдоним | Выполняемая команда |
|---|---|
| `makosh-nextest` | `nextest run --manifest-path backend/Cargo.toml` |
| `makosh-nextest-ci` | `nextest run --manifest-path backend/Cargo.toml --profile ci` |
| `makosh-nextest-integration` | `nextest run --manifest-path backend/Cargo.toml --profile integration --tests` |
| `makosh-llvm-cov` | `llvm-cov --manifest-path backend/Cargo.toml` |

Все перечисленные псевдонимы указывают `--manifest-path backend/Cargo.toml`.
```

### Source coverage / Покрытие источников

| Исходный файл | Покрытые факты |
|---|---|
| `.cargo/config.toml` | определённые псевдонимы `makosh-nextest`, `makosh-nextest-ci`, `makosh-nextest-integration`, `makosh-llvm-cov` и соответствующие им команды |

### Drift candidates / Кандидаты на drift

Из предоставленного контекста расхождения кода, документации или ADR не видны. Других источников для сравнения не встроено.
