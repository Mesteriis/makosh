### Summary / Резюме

В русскую wiki необходимо добавить (или обновить) страницу `operations/configuration.md`, документирующую файл `.config/nextest.toml`.
Файл содержит требуемую версию nextest, базовый профиль `default` и два дополнительных — `ci` и `integration`, с настройками повторных запусков, таймаутов, уровней вывода статусов и параметрами JUnit-отчётов.
Все утверждения в предлагаемой странице напрямую подкреплены содержимым исходного файла.

### Proposed pages / Предлагаемые страницы

`operations/configuration.md`:

```markdown
# Конфигурация nextest

Файл `.config/nextest.toml` задаёт поведение инструмента nextest.
Требуемая версия: `0.9.131`, рекомендуемая: `0.9.138`.

## Профили

Все профили используют `fail-fast = false`.

### `default`

Базовый профиль, применяемый при локальном запуске без указания другого профиля.

| Параметр             | Значение                                                       |
| -------------------- | -------------------------------------------------------------- |
| `fail-fast`          | `false`                                                        |
| `retries`            | `{ backoff = "fixed", count = 1, delay = "1s" }`               |
| `slow-timeout`       | `{ period = "45s", terminate-after = 3, grace-period = "10s" }` |
| `leak-timeout`       | `{ period = "500ms", result = "fail" }`                        |
| `status-level`       | `"slow"`                                                       |
| `final-status-level` | `"slow"`                                                       |
| `failure-output`     | `"immediate-final"`                                            |
| `success-output`     | `"never"`                                                      |

#### JUnit-отчёт

| Параметр                | Значение                     |
| ----------------------- | ---------------------------- |
| `path`                  | `"junit.xml"`                |
| `report-name`           | `"makosh-backend-default"`   |
| `store-success-output`  | `false`                      |
| `store-failure-output`  | `true`                       |

### `ci`

Профиль для непрерывной интеграции. Наследует `default` (`inherits = "default"`).

Отличия от `default`:

| Параметр             | Значение                                                       |
| -------------------- | -------------------------------------------------------------- |
| `retries`            | `{ backoff = "fixed", count = 2, delay = "1s" }`               |
| `slow-timeout`       | `{ period = "60s", terminate-after = 4, grace-period = "15s" }` |
| `status-level`       | `"retry"`                                                      |
| `final-status-level` | `"flaky"`                                                      |

#### JUnit-отчёт

| Параметр      | Значение                |
| ------------- | ----------------------- |
| `path`        | `"junit.xml"`           |
| `report-name` | `"makosh-backend-ci"`   |

Остальные параметры JUnit-отчёта унаследованы от `default`.

### `integration`

Профиль для интеграционных тестов. Наследует `default` (`inherits = "default"`).

Отличия от `default`:

| Параметр             | Значение                                                       |
| -------------------- | -------------------------------------------------------------- |
| `retries`            | `{ backoff = "fixed", count = 1, delay = "2s" }`               |
| `slow-timeout`       | `{ period = "120s", terminate-after = 2, grace-period = "15s" }` |
| `status-level`       | `"retry"`                                                      |
| `final-status-level` | `"slow"`                                                       |

#### JUnit-отчёт

| Параметр      | Значение                       |
| ------------- | ------------------------------ |
| `path`        | `"junit.xml"`                  |
| `report-name` | `"makosh-backend-integration"` |

Остальные параметры JUnit-отчёта унаследованы от `default`.
```

### Source coverage / Покрытие источников

| Исходный файл        | Факты, покрытые страницей                                                                                |
| -------------------- | -------------------------------------------------------------------------------------------------------- |
| `.config/nextest.toml` | Версия nextest (`nextest-version`), профиль `default` со всеми ключами, профили `ci` и `integration` с наследованием и переопределёнными ключами, секции JUnit-отчёта для каждого профиля. |

### Drift candidates / Кандидаты на drift

В предоставленном контексте расхождений между кодом и документацией не обнаружено — исходный файл один, иного эталонного материала для сравнения нет.
