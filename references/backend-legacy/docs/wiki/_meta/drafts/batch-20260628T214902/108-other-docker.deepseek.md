### Summary / Резюме

Страница `components/docker.md` должна быть создана или обновлена на основе встроенного Dockerfile и файла `.gitkeep`. На странице будет описана структура Docker-образов (dev, backend-builder, whatsapp-business-cloud-edge-proxy), команды сборки и рантайма, а также назначение директории `docker/data/`.

### Proposed pages / Предлагаемые страницы

#### `components/docker.md`

```markdown
# Docker

## Образы

Dockerfile определяет multi-stage сборку из трёх стадий.

### Стадия разработки (`dev`)

- Базовый образ: `rust:1.89-bookworm`
- Переменная окружения: `DEBIAN_FRONTEND=noninteractive`
- Аргумент сборки: `NODE_MAJOR=22`
- Компоненты Rust: `rustfmt`, `clippy`
- Системные пакеты (установлены через `apt-get`): `bash`, `ca-certificates`, `curl`, `git`, `gnupg`, `make`, `pkg-config`, `libssl-dev`, `protobuf-compiler`
- Node.js: мажорная версия `22` из репозитория `nodesource`, включён `corepack`
- Рабочая директория: `/workspace`
- Команда по умолчанию: `bash`

### Стадия сборки бэкенда (`backend-builder`)

- Наследует стадию `dev`
- Копируемые файлы: `Cargo.toml`, `Cargo.lock`, директории `backend/` и `crates/`
- Команда сборки:
  ```
  cargo build \
      --manifest-path backend/Cargo.toml \
      --bin makosh-whatsapp-business-cloud-edge-proxy \
      --release
  ```

### Стадия рантайма (`whatsapp-business-cloud-edge-proxy`)

- Базовый образ: `debian:bookworm-slim`
- Переменная окружения: `DEBIAN_FRONTEND=noninteractive`
- Системные пакеты: `ca-certificates`, `curl`
- Копирование артефакта:
  ```
  COPY --from=backend-builder \
      /workspace/target/release/makosh-whatsapp-business-cloud-edge-proxy \
      /usr/local/bin/makosh-whatsapp-business-cloud-edge-proxy
  ```
- EXPOSE порт: `8787`
- Команда запуска: `makosh-whatsapp-business-cloud-edge-proxy`

## Прочие файлы

### `docker/data/.gitkeep`

- Файл нулевого размера (1 байт, без содержимого). Вероятно, используется для сохранения директории `docker/data/` в Git. Более точное назначение из данного контекста не подтверждено.
```

### Source coverage / Покрытие источников

- `docker/Dockerfile`: описаны все три стадии сборки (`dev`, `backend-builder`, `whatsapp-business-cloud-edge-proxy`), их базовые образы, аргументы, системные пакеты, компоненты Rust, версия Node.js, команды копирования и сборки, порт EXPOSE, команда запуска.
- `docker/data/.gitkeep`: зафиксировано наличие пустого файла и отмечено, что его назначение не подтверждено контекстом.

### Drift candidates / Кандидаты на drift

Из предоставленного контекста расхождений не видно (отсутствуют другие исходные файлы, документация или ADR для сравнения).
