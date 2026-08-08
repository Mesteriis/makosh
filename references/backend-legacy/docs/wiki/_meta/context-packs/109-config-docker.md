# Задача для DeepSeek: обновить русскую Obsidian wiki

## Safety instructions / Инструкции безопасности

- Do not print, infer, summarize, or request secrets. / Не печатай, не выводи, не пересказывай и не запрашивай секреты.
- Treat `.env`, credential, token, key, certificate, and private paths as redacted even if referenced. / Считай `.env`, учетные данные, токены, ключи, сертификаты и приватные пути редактированными.
- Keep code identifiers, file paths, commands, package names, API names, and ADR titles exactly as written. / Сохраняй идентификаторы кода, пути, команды, имена пакетов, API и названия ADR без изменений.
- Write wiki prose in Russian and keep Markdown Obsidian-compatible. / Пиши текст wiki на русском и сохраняй совместимость с Obsidian Markdown.
- Do not invent source facts. If the context is insufficient, state that explicitly. / Не выдумывай факты об исходниках. Если контекста недостаточно, напиши это явно.
- Every behavioral statement in proposed wiki pages must be directly supported by the embedded source text. / Каждое утверждение о поведении в предлагаемых wiki-страницах должно напрямую подтверждаться встроенным текстом исходников.
- Do not infer semantics for profiles, flags, annotations, environment variables, or framework conventions unless this context pack explicitly defines them. / Не выводи семантику профилей, флагов, аннотаций, переменных окружения или framework-конвенций, если этот context pack явно её не определяет.
- Do not add external background knowledge about tools, frameworks, or CLIs. / Не добавляй внешние справочные знания об инструментах, framework или CLI.
- When only a command or config value is visible, document only the literal command or value. For deeper meaning, write only that it is not confirmed by this context. / Когда видна только команда или значение конфигурации, документируй только буквальную команду или значение. Для более глубокого смысла пиши только, что он не подтвержден этим контекстом.
- Do not name likely related files unless they are embedded in this context pack. / Не называй вероятные связанные файлы, если они не встроены в этот context pack.
- Use only the embedded Source Files section below. Do not call tools, read files, inspect the filesystem, or access MCP/web resources. / Используй только встроенный ниже раздел Source Files. Не вызывай tools, не читай файлы, не инспектируй файловую систему и не обращайся к MCP/web ресурсам.
- If a referenced path or wiki page is not embedded in this context pack, report insufficient context instead of trying to open it. / Если упомянутый путь или wiki-страница не встроены в этот context pack, укажи недостаток контекста вместо попытки открыть файл.

## Chunk details / Детали чанка

- Chunk ID / ID чанка: `109-config-docker`
- Group / Группа: `docker`
- Role / Роль: `config`
- Status / Статус: `pending`
- Repository / Репозиторий: `/Users/avm/projects/Personal/makosh`
- Wiki path / Путь wiki: `/Users/avm/projects/Personal/makosh/docs/wiki`
- Metadata path / Путь metadata: `/Users/avm/projects/Personal/makosh/docs/wiki/_meta`
- Plan generated at / План создан: `2026-06-28T19:48:55Z`
- Per-file source limit / Лимит источника на файл: `12000` characters

## Target pages / Целевые страницы

- `operations/configuration.md`

## Required Output / Требуемый результат

Return one Markdown response with these sections and no extra wrapper text. / Верни один Markdown-ответ с этими разделами и без дополнительной обертки.

### Summary / Резюме

Briefly describe what should change in the Russian wiki and why. / Кратко опиши, что нужно изменить в русской wiki и почему.

### Proposed pages / Предлагаемые страницы

For each target page, provide the wiki-relative path and full proposed Obsidian-compatible Markdown content. / Для каждой целевой страницы укажи путь относительно wiki и полный предложенный Markdown, совместимый с Obsidian.

### Source coverage / Покрытие источников

List each source file and the facts from it that the proposed pages cover. / Перечисли каждый исходный файл и факты из него, покрытые предложенными страницами.

### Drift candidates / Кандидаты на drift

List possible code/docs/ADR drift found in this chunk, or state that none is visible from the provided context. / Перечисли возможные расхождения кода, документации и ADR в этом чанке либо укажи, что из данного контекста они не видны.

## Source Files / Исходные файлы

### `docker/docker-compose.yml`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docker/docker-compose.yml`
- Size bytes / Размер в байтах: `2475`
- Included characters / Включено символов: `2475`
- Truncated / Обрезано: `no`

```yaml
name: makosh-dev

services:
  postgres:
    image: pgvector/pgvector:0.8.2-pg16
    restart: unless-stopped
    environment:
      POSTGRES_DB: ${MAKOSH_POSTGRES_DB:?Set MAKOSH_POSTGRES_DB in docker/.env}
      POSTGRES_USER: ${MAKOSH_POSTGRES_USER:?Set MAKOSH_POSTGRES_USER in docker/.env}
      POSTGRES_PASSWORD: ${MAKOSH_POSTGRES_PASSWORD:?Set MAKOSH_POSTGRES_PASSWORD in docker/.env}
    ports:
      - "${MAKOSH_POSTGRES_BIND:-127.0.0.1}:${MAKOSH_POSTGRES_PORT:-5432}:5432"
    volumes:
      - type: bind
        source: ./data/postgres
        target: /var/lib/postgresql/data
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U \"$${POSTGRES_USER}\" -d \"$${POSTGRES_DB}\""]
      interval: 10s
      timeout: 5s
      retries: 5
      start_period: 10s

  nats:
    image: nats:2.11-alpine
    restart: unless-stopped
    command: ["-js", "-sd", "/data", "-m", "8222"]
    ports:
      - "${MAKOSH_NATS_BIND:-127.0.0.1}:${MAKOSH_NATS_PORT:-4222}:4222"
      - "${MAKOSH_NATS_MONITOR_BIND:-127.0.0.1}:${MAKOSH_NATS_MONITOR_PORT:-8222}:8222"
    volumes:
      - type: bind
        source: ./data/nats
        target: /data
    healthcheck:
      test: ["CMD", "wget", "-q", "-O", "-", "http://127.0.0.1:8222/healthz"]
      interval: 10s
      timeout: 5s
      retries: 5
      start_period: 5s

  whatsapp-business-cloud-edge-proxy:
    profiles:
      - whatsapp-business-cloud-edge
    build:
      context: ..
      dockerfile: docker/Dockerfile
      target: whatsapp-business-cloud-edge-proxy
    restart: unless-stopped
    environment:
      MAKOSH_WHATSAPP_BUSINESS_CLOUD_EDGE_BIND_ADDR: 0.0.0.0:8787
      MAKOSH_WHATSAPP_BUSINESS_CLOUD_EDGE_MAKOSH_BASE_URL: ${MAKOSH_WHATSAPP_BUSINESS_CLOUD_EDGE_MAKOSH_BASE_URL:-http://host.docker.internal:8080}
      MAKOSH_WHATSAPP_BUSINESS_CLOUD_EDGE_ACCOUNT_ID: ${MAKOSH_WHATSAPP_BUSINESS_CLOUD_EDGE_ACCOUNT_ID:-}
      MAKOSH_LOCAL_API_SECRET: ${MAKOSH_LOCAL_API_SECRET:?Set MAKOSH_LOCAL_API_SECRET in docker/.env}
      MAKOSH_LOG_FORMAT: ${MAKOSH_LOG_FORMAT:-json}
      RUST_LOG: ${MAKOSH_WHATSAPP_BUSINESS_CLOUD_EDGE_RUST_LOG:-info}
    ports:
      - "${MAKOSH_WHATSAPP_BUSINESS_CLOUD_EDGE_BIND:-127.0.0.1}:${MAKOSH_WHATSAPP_BUSINESS_CLOUD_EDGE_PORT:-8787}:8787"
    extra_hosts:
      - "host.docker.internal:host-gateway"
    healthcheck:
      test: ["CMD-SHELL", "curl -fsS http://127.0.0.1:8787/healthz >/dev/null"]
      interval: 10s
      timeout: 5s
      retries: 5
      start_period: 5s
```
