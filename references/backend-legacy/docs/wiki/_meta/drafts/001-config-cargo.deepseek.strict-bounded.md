### Summary / Резюме

Добавляется новая страница `operations/configuration.md` русской Obsidian‑wiki, документирующая содержимое `.cargo/config.toml` — файл с алиасами Cargo, используемыми в репозитории `makosh‑hub`. Это улучшает понимание доступных команд для запуска тестов и покрытия, а также делает конфигурацию явной для разработчиков.

### Proposed pages / Предлагаемые страницы

`operations/configuration.md`

```markdown
# Конфигурация .cargo

Файл `.cargo/config.toml` в корне репозитория `makosh‑hub` содержит алиасы для утилиты `cargo`, упрощающие запуск тестов и сбор покрытия кода.

## Алиасы

| Алиас | Команда |
|---|---|
| `makosh-nextest` | `nextest run --manifest-path backend/Cargo.toml` |
| `makosh-nextest-ci` | `nextest run --manifest-path backend/Cargo.toml --profile ci` |
| `makosh-nextest-integration` | `nextest run --manifest-path backend/Cargo.toml --profile integration --tests` |
| `makosh-llvm-cov` | `llvm-cov --manifest-path backend/Cargo.toml` |

## Пояснения

- **Общий параметр** – все алиасы передают `--manifest-path backend/Cargo.toml`, что направляет команду на пакет `backend`.
- **`makosh-nextest-ci`** – добавляет `--profile ci`. Предположительно, профиль `ci` определён в `.config/nextest.toml`; данный контекст не подтверждает наличие и содержимое этого файла.
- **`makosh-nextest-integration`** – использует `--profile integration` и флаг `--tests`. Флаг `--tests` в `cargo nextest` без дополнительного фильтра обычно запускает все тесты (кроме doc-тестов), однако точное поведение в данном окружении контекстом не подтверждено.
- **`makosh-llvm-cov`** – вызывает `llvm-cov`. Требуется установленный плагин `cargo‑llvm‑cov`; его наличие в окружении разработки данным контекстом не подтверждено.

> [!note] Не подтверждено контекстом
> Содержимое профилей `ci` и `integration`, точная семантика `--tests` без аргумента, а также доступность `cargo‑llvm‑cov` не верифицированы в рамках данного контекст‑пакета. Приведённые пояснения основаны только на буквальном содержимом `.cargo/config.toml`.
```

### Source coverage / Покрытие источников

- **`.cargo/config.toml`**
  - Путь к файлу: `/Users/avm/projects/Personal/makosh/.cargo/config.toml`
  - Факт: файл определяет секцию `[alias]` с четырьмя алиасами:
    - `makosh-nextest` → `nextest run --manifest-path backend/Cargo.toml`
    - `makosh-nextest-ci` → `nextest run --manifest-path backend/Cargo.toml --profile ci`
    - `makosh-nextest-integration` → `nextest run --manifest-path backend/Cargo.toml --profile integration --tests`
    - `makosh-llvm-cov` → `llvm-cov --manifest-path backend/Cargo.toml`
  - Факт: все алиасы адресуют манифест `backend/Cargo.toml`.

### Drift candidates / Кандидаты на drift

Из предоставленного контекста дрифт между кодом, документацией и архитектурными решениями не виден. Предыдущая версия wiki‑страницы `operations/configuration.md` отсутствует в контекст‑пакете; без неё сравнение невозможно, но предлагаемое содержимое полностью согласовано с единственным исходным файлом.
