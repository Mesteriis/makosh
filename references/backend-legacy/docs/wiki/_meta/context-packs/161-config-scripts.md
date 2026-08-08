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

- Chunk ID / ID чанка: `161-config-scripts`
- Group / Группа: `scripts`
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

### `scripts/architecture-contract.json`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/scripts/architecture-contract.json`
- Size bytes / Размер в байтах: `4043`
- Included characters / Включено символов: `4043`
- Truncated / Обрезано: `no`

```json
{
  "schema_version": 1,
  "adr": "docs/adr/ADR-0098-provider-neutral-communications-api-and-strict-boundaries.md",
  "interaction_kinds": [
    "direct_call",
    "command_port",
    "query_port",
    "event",
    "projection",
    "runtime_integration_api"
  ],
  "backend": {
    "layers": {
      "app": {
        "allow": [
          "domain_command_ports",
          "domain_query_ports",
          "integration_runtime_integration_api",
          "platform"
        ],
        "deny": [
          "stores",
          "business_orchestration"
        ]
      },
      "domains": {
        "owned": [
          "agents",
          "calendar",
          "communications",
          "decisions",
          "documents",
          "graph",
          "knowledge",
          "mail",
          "notes",
          "obligations",
          "organizations",
          "personas",
          "persons",
          "projects",
          "radar",
          "relationships",
          "signal_hub",
          "tasks",
          "timeline"
        ],
        "allow": [
          "own_modules",
          "platform",
          "pure_engines"
        ],
        "deny": [
          "other_domains",
          "integrations",
          "app",
          "workflows",
          "vault"
        ]
      },
      "integrations": {
        "allow": [
          "own_modules",
          "platform",
          "vault",
          "external_sdks"
        ],
        "deny": [
          "domains",
          "app",
          "workflows",
          "business_truth"
        ]
      },
      "workflows": {
        "allow": [
          "domain_command_ports",
          "domain_query_ports",
          "events",
          "platform"
        ],
        "deny": [
          "stores",
          "handlers",
          "integration_clients"
        ]
      },
      "engines": {
        "allow": [
          "own_projections",
          "own_indexes",
          "platform"
        ],
        "deny": [
          "business_domain_mutation",
          "integrations"
        ]
      },
      "ai": {
        "allow": [
          "candidates",
          "summaries",
          "classifications",
          "embeddings"
        ],
        "deny": [
          "domain_stores",
          "domain_mutation",
          "source_of_truth"
        ]
      },
      "platform": {
        "allow": [
          "neutral_contracts",
          "technical_infrastructure"
        ],
        "deny": [
          "domains",
          "integrations",
          "workflows",
          "business_table_sql"
        ]
      },
      "vault": {
        "allow": [
          "secrets",
          "sessions",
          "runtime_state"
        ],
        "deny": [
          "business_truth"
        ]
      }
    }
  },
  "frontend": {
    "layers": {
      "app": {
        "allow": [
          "compose_domain_views",
          "compose_domain_stores",
          "routing"
        ]
      },
      "domains": {
        "allow": [
          "own_modules",
          "platform",
          "shared"
        ],
        "deny": [
          "other_frontend_domains",
          "provider_runtime_state",
          "integrations"
        ]
      },
      "integrations": {
        "allow": [
          "provider_setup",
          "provider_runtime",
          "platform",
          "shared"
        ],
        "deny": [
          "business_truth",
          "domains"
        ]
      }
    },
    "business_route_model": {
      "business_root": "/api/v1/communications",
      "forbidden_provider_business_roots": [
        "/api/v1/integrations/mail/*",
        "/api/v1/integrations/telegram/*",
        "/api/v1/integrations/whatsapp/*"
      ],
      "runtime_root": "/api/v1/integrations/{provider}"
    },
    "provider_business_cache_roots": {
      "forbidden": [
        "telegram",
        "whatsapp",
        "mail"
      ],
      "business_query_key_root": "communications",
      "runtime_query_key_root": [
        "integrations",
        "<provider>",
        "runtime"
      ]
    }
  }
}
```
