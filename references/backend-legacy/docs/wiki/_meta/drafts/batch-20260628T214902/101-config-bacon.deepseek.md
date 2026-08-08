### Summary / Резюме

Создаётся или обновляется страница `operations/configuration.md` русской Obsidian‑wiki для проекта `makosh‑hub`. На основе встроенного файла `bacon.toml` документируется задание `backend-dev` инструмента bacon: его команда, отслеживаемые пути и флаги `allow_warnings`/`show_command_error_code`. Страница отражает только буквальное содержимое этого файла, без додумывания внешних конвенций.

### Proposed pages / Предлагаемые страницы

#### `operations/configuration.md`

````markdown
# Конфигурация bacon

Файл `bacon.toml` в корне репозитория определяет задания для инструмента bacon.

## Задание `backend-dev`

```toml
[jobs.backend-dev]
command = ["cargo", "run", "--manifest-path", "backend/Cargo.toml"]
watch = ["backend/src", "backend/migrations", "backend/Cargo.toml", "backend/Cargo.lock"]
allow_warnings = true
show_command_error_code = true
```

- **Команда:** выполняет `cargo run` с указанием манифеста `backend/Cargo.toml`.
- **Отслеживаемые пути:** задание автоматически запускается при изменениях в каталогах и файлах:
  - `backend/src`
  - `backend/migrations`
  - `backend/Cargo.toml`
  - `backend/Cargo.lock`
- **`allow_warnings = true`** – предупреждения компилятора не считаются ошибкой задания.
- **`show_command_error_code = true`** – код возврата команды (`cargo run`) отображается в случае неуспешного завершения.

> Дополнительная семантика ключей `allow_warnings` и `show_command_error_code` не подтверждается данным контекстом.
````

### Source coverage / Покрытие источников

- **`bacon.toml`**
  - Имя задания: `backend-dev` в секции `[jobs]`.
  - Поле `command` и его значение.
  - Поле `watch` и список отслеживаемых путей.
  - Флаг `allow_warnings` (значение `true`).
  - Флаг `show_command_error_code` (значение `true`).

Все существенные факты из файла перенесены в предложенную страницу без изменений.

### Drift candidates / Кандидаты на drift

Присутствующий контекст содержит только один исходный файл. Других версий конфигурации, документации или ADR не предоставлено, поэтому расхождения (drift) между кодом и документацией не видны.
