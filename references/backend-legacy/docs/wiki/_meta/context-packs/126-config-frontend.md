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

- Chunk ID / ID чанка: `126-config-frontend`
- Group / Группа: `frontend`
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

### `frontend/.gitignore`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/.gitignore`
- Size bytes / Размер в байтах: `161`
- Included characters / Включено символов: `161`
- Truncated / Обрезано: `no`

```text
node_modules

# Output
/dist

# OS
.DS_Store
Thumbs.db

# Env
.env
.env.*
!.env.example
!.env.test

# Vite
vite.config.js.timestamp-*
vite.config.ts.timestamp-*
```

### `frontend/package.json`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/package.json`
- Size bytes / Размер в байтах: `1633`
- Included characters / Включено символов: `1633`
- Truncated / Обрезано: `no`

```json
{
	"name": "frontend",
	"private": true,
	"version": "0.0.1",
	"type": "module",
	"packageManager": "pnpm@11.5.1",
	"scripts": {
		"dev": "vite",
		"proto:generate": "node scripts/generate-proto.mjs",
		"build": "pnpm typecheck && vite build",
		"typecheck": "vue-tsc --noEmit",
		"lint:ox": "oxlint --vue-plugin .",
		"lint:styles": "node scripts/check-no-inline-styles.mjs",
		"lint:srp": "node scripts/check-component-lines.mjs",
		"lint": "pnpm lint:ox && pnpm lint:styles && pnpm lint:srp",
		"preview": "vite preview",
		"test": "pnpm test:unit",
		"test:unit": "vitest run",
		"validate": "pnpm lint && pnpm typecheck && pnpm test:unit && pnpm build"
	},
	"devDependencies": {
		"@bufbuild/protoc-gen-es": "^2.12.0",
		"@tauri-apps/cli": "^2.11.2",
		"@types/node": "^25.9.1",
		"@vitejs/plugin-vue": "^6.0.7",
		"autoprefixer": "^10.4.20",
		"oxlint": "^1.70.0",
		"postcss": "^8.5.3",
		"tailwindcss": "^3.4.17",
		"typescript": "^6.0.2",
		"vite": "^8.0.7",
		"vitest": "^4.1.8",
		"vue-tsc": "^2.2.8"
	},
	"dependencies": {
		"@bufbuild/protobuf": "^2.12.0",
		"@connectrpc/connect": "^2.1.2",
		"@connectrpc/connect-web": "^2.1.2",
		"@iconify/vue": "^4.3.0",
		"@tanstack/vue-query": "^5.101.0",
		"@tanstack/vue-table": "^8.21.3",
		"@tanstack/vue-virtual": "^3.13.28",
		"@tauri-apps/api": "^2.11.1",
		"@tiptap/vue-3": "^3.26.1",
		"@vee-validate/zod": "^4.15.1",
		"@vue-flow/core": "^1.48.2",
		"@vueuse/core": "^14.3.0",
		"date-fns": "^4.4.0",
		"motion-v": "^2.3.0",
		"pinia": "^3.0.4",
		"reka-ui": "^2.9.10",
		"vee-validate": "^4.15.1",
		"vue": "^3.5.38",
		"vue-router": "^4.5.0",
		"zod": "^3.25.76"
	}
}
```

### `frontend/pnpm-lock.yaml`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/pnpm-lock.yaml`
- Size bytes / Размер в байтах: `95613`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```yaml
lockfileVersion: '9.0'

settings:
  autoInstallPeers: true
  excludeLinksFromLockfile: false

importers:

  .:
    dependencies:
      '@bufbuild/protobuf':
        specifier: ^2.12.0
        version: 2.12.0
      '@connectrpc/connect':
        specifier: ^2.1.2
        version: 2.1.2(@bufbuild/protobuf@2.12.0)
      '@connectrpc/connect-web':
        specifier: ^2.1.2
        version: 2.1.2(@bufbuild/protobuf@2.12.0)(@connectrpc/connect@2.1.2(@bufbuild/protobuf@2.12.0))
      '@iconify/vue':
        specifier: ^4.3.0
        version: 4.3.0(vue@3.5.38(typescript@6.0.3))
      '@tanstack/vue-query':
        specifier: ^5.101.0
        version: 5.101.0(vue@3.5.38(typescript@6.0.3))
      '@tanstack/vue-table':
        specifier: ^8.21.3
        version: 8.21.3(vue@3.5.38(typescript@6.0.3))
      '@tanstack/vue-virtual':
        specifier: ^3.13.28
        version: 3.13.28(vue@3.5.38(typescript@6.0.3))
      '@tauri-apps/api':
        specifier: ^2.11.1
        version: 2.11.1
      '@tiptap/vue-3':
        specifier: ^3.26.1
        version: 3.26.1(@floating-ui/dom@1.7.6)(@tiptap/core@3.26.1(@tiptap/pm@3.26.1))(@tiptap/pm@3.26.1)(vue@3.5.38(typescript@6.0.3))
      '@vee-validate/zod':
        specifier: ^4.15.1
        version: 4.15.1(vue@3.5.38(typescript@6.0.3))(zod@3.25.76)
      '@vue-flow/core':
        specifier: ^1.48.2
        version: 1.48.2(vue@3.5.38(typescript@6.0.3))
      '@vueuse/core':
        specifier: ^14.3.0
        version: 14.3.0(vue@3.5.38(typescript@6.0.3))
      date-fns:
        specifier: ^4.4.0
        version: 4.4.0
      motion-v:
        specifier: ^2.3.0
        version: 2.3.0(@vueuse/core@14.3.0(vue@3.5.38(typescript@6.0.3)))(vue@3.5.38(typescript@6.0.3))
      pinia:
        specifier: ^3.0.4
        version: 3.0.4(typescript@6.0.3)(vue@3.5.38(typescript@6.0.3))
      reka-ui:
        specifier: ^2.9.10
        version: 2.9.10(vue@3.5.38(typescript@6.0.3))
      vee-validate:
        specifier: ^4.15.1
        version: 4.15.1(vue@3.5.38(typescript@6.0.3))
      vue:
        specifier: ^3.5.38
        version: 3.5.38(typescript@6.0.3)
      vue-router:
        specifier: ^4.5.0
        version: 4.6.4(vue@3.5.38(typescript@6.0.3))
      zod:
        specifier: ^3.25.76
        version: 3.25.76
    devDependencies:
      '@bufbuild/protoc-gen-es':
        specifier: ^2.12.0
        version: 2.12.0(@bufbuild/protobuf@2.12.0)
      '@tauri-apps/cli':
        specifier: ^2.11.2
        version: 2.11.2
      '@types/node':
        specifier: ^25.9.1
        version: 25.9.1
      '@vitejs/plugin-vue':
        specifier: ^6.0.7
        version: 6.0.7(vite@8.0.16(@types/node@25.9.1)(jiti@1.21.7))(vue@3.5.38(typescript@6.0.3))
      autoprefixer:
        specifier: ^10.4.20
        version: 10.5.0(postcss@8.5.15)
      oxlint:
        specifier: ^1.70.0
        version: 1.70.0
      postcss:
        specifier: ^8.5.3
        version: 8.5.15
      tailwindcss:
        specifier: ^3.4.17
        version: 3.4.19
      typescript:
        specifier: ^6.0.2
        version: 6.0.3
      vite:
        specifier: ^8.0.7
        version: 8.0.16(@types/node@25.9.1)(jiti@1.21.7)
      vitest:
        specifier: ^4.1.8
        version: 4.1.8(@types/node@25.9.1)(vite@8.0.16(@types/node@25.9.1)(jiti@1.21.7))
      vue-tsc:
        specifier: ^2.2.8
        version: 2.2.12(typescript@6.0.3)

packages:

  '@alloc/quick-lru@5.2.0':
    resolution: {integrity: sha512-UrcABB+4bUrFABwbluTIBErXwvbsU/V7TZWfmbgJfbkwiBuziS9gxdODUyuiecfdGQ85jglMW6juS3+z5TsKLw==}
    engines: {node: '>=10'}

  '@babel/helper-string-parser@7.29.7':
    resolution: {integrity: sha512-Pb5ijPrZ89GDH8223L4UP8i6QApWxs04RbPQJTeWDV0/keR2E36MeKnyr6LYmUUvqRRI+Iv87SuF1W6ErINzYw==}
    engines: {node: '>=6.9.0'}

  '@babel/helper-validator-identifier@7.29.7':
    resolution: {integrity: sha512-qehxGkRj55h/ff8EMaJ+cYhyaKlHIxqYDn682wQD7RNp9UujOQsHog2uS0r2vzr4pW+sXf90NeeayjcNaX3fFg==}
    engines: {node: '>=6.9.0'}

  '@babel/parser@7.29.7':
    resolution: {integrity: sha512-hnORnjP/1P/zFEndoeX+n+t1RwWRJiJpM/jO7FW32Kn9r5+sJB2JWOdYo4L6k78j15eCwY3Gm/7364B1EMwtNg==}
    engines: {node: '>=6.0.0'}
    hasBin: true

  '@babel/types@7.29.7':
    resolution: {integrity: sha512-4zBIxpPzowiZpusoFkyGVwakdRJUyuH5PxQ/PrqghfdFWWasvnCdPfQXHrenDai+gyLARulZjZowCOj6fjT4pA==}
    engines: {node: '>=6.9.0'}

  '@bufbuild/protobuf@2.12.0':
    resolution: {integrity: sha512-B/XlCaFIP8LOwzo+bz5uFzATYokcwCKQcghqnlfwSmM5eX/qTkvDBnDPs+gXtX/RyjxJ4DRikECcPJbyALA8FA==}

  '@bufbuild/protoc-gen-es@2.12.0':
    resolution: {integrity: sha512-d9htF6jEkSwPbp9d/vSmZOBF7eeG18AvTMKmVg4I23afnrQOxL2w3WOXa9TaufMCyu24QakEUb4vux8apI5e7A==}
    engines: {node: '>=20'}
    hasBin: true
    peerDependencies:
      '@bufbuild/protobuf': 2.12.0
    peerDependenciesMeta:
      '@bufbuild/protobuf':
        optional: true

  '@bufbuild/protoplugin@2.12.0':
    resolution: {integrity: sha512-ORlDITp8AFUXzIhLRoMCG+ud+D3MPKWb5HQXBoskMMnjeyEjE1H1qLonVNPyOr8lkx3xSfYUo8a0dvOZJVAzow==}

  '@connectrpc/connect-web@2.1.2':
    resolution: {integrity: sha512-1tfaK85MU+gJjwwmL31d2rzdf0XCYX99chZf63uG89SGBUd4XuZ4ZzhGo2u79TPXOE6nLIZQ2okrpyey42PYdg==}
    peerDependencies:
      '@bufbuild/protobuf': ^2.7.0
      '@connectrpc/connect': 2.1.2

  '@connectrpc/connect@2.1.2':
    resolution: {integrity: sha512-MXkBijtcX09R10Eb6sFeIetc6w6746eio6xtfuyVOH7oQAacT1X0GzMIQFux6Qy8cq3W/T5qX5Bei8YbFtmRGA==}
    peerDependencies:
      '@bufbuild/protobuf': ^2.7.0

  '@emnapi/core@1.10.0':
    resolution: {integrity: sha512-yq6OkJ4p82CAfPl0u9mQebQHKPJkY7WrIuk205cTYnYe+k2Z8YBh11FrbRG/H6ihirqcacOgl2BIO8oyMQLeXw==}

  '@emnapi/runtime@1.10.0':
    resolution: {integrity: sha512-ewvYlk86xUoGI0zQRNq/mC+16R1QeDlKQy21Ki3oSYXNgLb45GV1P6A0M+/s6nyCuNDqe5VpaY84BzXGwVbwFA==}

  '@emnapi/wasi-threads@1.2.1':
    resolution: {integrity: sha512-uTII7OYF+/Mes/MrcIOYp5yOtSMLBWSIoLPpcgwipoiKbli6k322tcoFsxoIIxPDqW01SQGAgko4EzZi2BNv2w==}

  '@floating-ui/core@1.7.5':
    resolution: {integrity: sha512-1Ih4WTWyw0+lKyFMcBHGbb5U5FtuHJuujoyyr5zTaWS5EYMeT6Jb2AuDeftsCsEuchO+mM2ij5+q9crhydzLhQ==}

  '@floating-ui/dom@1.7.6':
    resolution: {integrity: sha512-9gZSAI5XM36880PPMm//9dfiEngYoC6Am2izES1FF406YFsjvyBMmeJ2g4SAju3xWwtuynNRFL2s9hgxpLI5SQ==}

  '@floating-ui/utils@0.2.11':
    resolution: {integrity: sha512-RiB/yIh78pcIxl6lLMG0CgBXAZ2Y0eVHqMPYugu+9U0AeT6YBeiJpf7lbdJNIugFP5SIjwNRgo4DhR1Qxi26Gg==}

  '@floating-ui/vue@1.1.11':
    resolution: {integrity: sha512-HzHKCNVxnGS35r9fCHBc3+uCnjw9IWIlCPL683cGgM9Kgj2BiAl8x1mS7vtvP6F9S/e/q4O6MApwSHj8hNLGfw==}

  '@iconify/types@2.0.0':
    resolution: {integrity: sha512-+wluvCrRhXrhyOmRDJ3q8mux9JkKy5SJ/v8ol2tu4FVjyYvtEzkc/3pK15ET6RKg4b4w4BmTk1+gsCUhf21Ykg==}

  '@iconify/vue@4.3.0':
    resolution: {integrity: sha512-Xq0h6zMrHBbrW8jXJ9fISi+x8oDQllg5hTDkDuxnWiskJ63rpJu9CvJshj8VniHVTbsxCg9fVoPAaNp3RQI5OQ==}
    peerDependencies:
      vue: '>=3'

  '@internationalized/date@3.12.2':
    resolution: {integrity: sha512-FY1Y+H64NDs+HAF6omlnWxm3mEpfgaCSWtL5l551ZZfImA+kGjPFgrnJrGjH6lfmLL0g8Z/mBu1R3kufeCp6Jw==}

  '@internationalized/number@3.6.7':
    resolution: {integrity: sha512-3ji1fcrT+FPAK86UqEhB/psHixYo6niWPJtt7+qRaYFynt/BaJG8GhAPimtWUpEiVSTq8ZM8L5psMxGquiB/Vg==}

  '@jridgewell/gen-mapping@0.3.13':
    resolution: {integrity: sha512-2kkt/7niJ6MgEPxF0bYdQ6etZaA+fQvDcLKckhy1yIQOzaoKjBBjSj63/aLVjYE3qhRt5dvM+uUyfCg6UKCBbA==}

  '@jridgewell/resolve-uri@3.1.2':
    resolution: {integrity: sha512-bRISgCIjP20/tbWSPWMEi54QVPRZExkuD9lJL+UIxUKtwVJA8wW1Trb1jMs1RFXo1CBTNZ/5hpC9QvmKWdopKw==}
    engines: {node: '>=6.0.0'}

  '@jridgewell/sourcemap-codec@1.5.5':
    resolution: {integrity: sha512-cYQ9310grqxueWbl+WuIUIaiUaDcj7WOq5fVhEljNVgRfOUhY9fy2zTvfoqWsnebh8Sl70VScFbICvJnLKB0Og==}

  '@jridgewell/trace-mapping@0.3.31':
    resolution: {integrity: sha512-zzNR+SdQSDJzc8joaeP8QQoCQr8NuYx2dIIytl1QeBEZHJ9uW6hebsrYgbz8hJwUQao3TWCMtmfV8Nu1twOLAw==}

  '@napi-rs/wasm-runtime@1.1.4':
    resolution: {integrity: sha512-3NQNNgA1YSlJb/kMH1ildASP9HW7/7kYnRI2szWJaofaS1hWmbGI4H+d3+22aGzXXN9IJ+n+GiFVcGipJP18ow==}
    peerDependencies:
      '@emnapi/core': ^1.7.1
      '@emnapi/runtime': ^1.7.1

  '@nodelib/fs.scandir@2.1.5':
    resolution: {integrity: sha512-vq24Bq3ym5HEQm2NKCr3yXDwjc7vTsEThRDnkp2DK9p1uqLR+DHurm/NOTo0KG7HYHU7eppKZj3MyqYuMBf62g==}
    engines: {node: '>= 8'}

  '@nodelib/fs.stat@2.0.5':
    resolution: {integrity: sha512-RkhPPp2zrqDAQA/2jNhnztcPAlv64XdhIp7a7454A5ovI7Bukxgt7MX7udwAu3zg1DcpPU0rz3VV1SeaqvY4+A==}
    engines: {node: '>= 8'}

  '@nodelib/fs.walk@1.2.8':
    resolution: {integrity: sha512-oGB+UxlgWcgQkgwo8GcEGwemoTFt3FIO9ababBmaGwXIoBKZ+GTy0pP185beGg7Llih/NSHSV2XAs1lnznocSg==}
    engines: {node: '>= 8'}

  '@oxc-project/types@0.133.0':
    resolution: {integrity: sha512-KzkdCd6Uxqnf6l3HOw1xfatAlUURA0g14cvBYFyJ5SaNOQbOUvBr9PKArcPcrNIeRsBdgcUzOGrhKveVpvOIGA==}

  '@oxlint/binding-android-arm-eabi@1.70.0':
    resolution: {integrity: sha512-zFh0P4cswmRvw6nkyb89dr18rRanuaCPAsEXsFDoQY8WdaquI8Pt4NWFjaMJg6L23cy5NeN8J9cBnREbWzZhaw==}
    engines: {node: ^20.19.0 || >=22.12.0}
    cpu: [arm]
    os: [android]

  '@oxlint/binding-android-arm64@1.70.0':
    resolution: {integrity: sha512-qI8o4HZjeGiBrWv+pJv4lH0Yi2Gl/JSp/EumBUApezJprIKa5PS4nU0lQsQngtky8k+SplQIOjv6hwu0SSxeyg==}
    engines: {node: ^20.19.0 || >=22.12.0}
    cpu: [arm64]
    os: [android]

  '@oxlint/binding-darwin-arm64@1.70.0':
    resolution: {integrity: sha512-8KjgVVHI5F9nVwHCRwwA78Ty7zNKP4Wd9OeN5PSv3iu/F/u1RVXoOCgLhWqust6HmwQG6xc8c+RCyaWENy24+w==}
    engines: {node: ^20.19.0 || >=22.12.0}
    cpu: [arm64]
    os: [darwin]

  '@oxlint/binding-darwin-x64@1.70.0':
    resolution: {integrity: sha512-WVydssv5PSUBXFJTdNBWlmGkbNmvPGaFt/2SUT/EZRB6bq6bEOHmMlbnupZD5jmlEvi9+mZJHi8TCw15lyfSfQ==}
    engines: {node: ^20.19.0 || >=22.12.0}
    cpu: [x64]
    os: [darwin]

  '@oxlint/binding-freebsd-x64@1.70.0':
    resolution: {integrity: sha512-hJucmUf8OlinHNb1R7fI4Fw6WsAstOz7i8nmkWQfiHoZXtbufNm+MxiDTIMk1ggh2Ro4vLzgQ+bKvRY54MZoRA==}
    engines: {node: ^20.19.0 || >=22.12.0}
    cpu: [x64]
    os: [freebsd]

  '@oxlint/binding-linux-arm-gnueabihf@1.70.0':
    resolution: {integrity: sha512-1BnS7wbCYDSXwWzJJ+mc3NURoha6m6m6RT5c6vgAY3oz7C3OVXP+S0awo2mRq97arrJkVvO3qRQfyAHL+76xtQ==}
    engines: {node: ^20.19.0 || >=22.12.0}
    cpu: [arm]
    os: [linux]

  '@oxlint/binding-linux-arm-musleabihf@1.70.0':
    resolution: {integrity: sha512-yKy/UdbR55+M2yEcuiV5DCNC/gdQAjr/GioUy50QwBzSrKm8ueWADqyRLS9Xk+qjNeCYGg6A8FvUBds56ttfqg==}
    engines: {node: ^20.19.0 || >=22.12.0}
    cpu: [arm]
    os: [linux]

  '@oxlint/binding-linux-arm64-gnu@1.70.0':
    resolution: {integrity: sha512-0A5XJ4alvmqFUFP/4oYSyaO+qLto/HrKEWTSaegiVl+HOufFngK2BjYw9x4RbwBt/du5QG6l5q1zeWiJYYG5yg==}
    engines: {node: ^20.19.0 || >=22.12.0}
    cpu: [arm64]
    os: [linux]
    libc: [glibc]

  '@oxlint/binding-linux-arm64-musl@1.70.0':
    resolution: {integrity: sha512-JiylyurlB0CLSedNtx1gzv3FvfWPF1h/2Y3BJszPLNt5XQFlBsH5ke0Jle3iJb3uqu5m2e7A/DwzpuCAHdiU+A==}
    engines: {node: ^20.19.0 || >=22.12.0}
    cpu: [arm64]
    os: [linux]
    libc: [musl]

  '@oxlint/binding-linux-ppc64-gnu@1.70.0':
    resolution: {integrity: sha512-J8VPG7I3/HmgaU4u8pNU2kFx2+0U+vPLS1dXFxXOaR/2TQ0f8AC7DRz0SRGRI1bfphnX2hVYTTtLuhL4nYKL+Q==}
    engines: {node: ^20.19.0 || >=22.12.0}
    cpu: [ppc64]
    os: [linux]
    libc: [glibc]

  '@oxlint/binding-linux-riscv64-gnu@1.70.0':
    resolution: {integrity: sha512-N2+4lV2KLN+oXTIIIwmWDhwkrnvqf5oX7Hw0zPjk+RuIVgiBQSOlJWF7uQoFx2siEYX0ZQ5cfSbEAHm+J3t7Wg==}
    engines: {node: ^20.19.0 || >=22.12.0}
    cpu: [riscv64]
    os: [linux]
    libc: [glibc]

  '@oxlint/binding-linux-riscv64-musl@1.70.0':
    resolution: {integrity: sha512-1e2L7cFCvx9QDzq6NPP+0tABKb5z6nWHyddWTNKprEsjO9xNrAtPowuCGpjNXxkTdsMiZ4jc8YQ5SstZd4XK6g==}
    engines: {node: ^20.19.0 || >=22.12.0}
    cpu: [riscv64]
    os: [linux]
    libc: [musl]

  '@oxlint/binding-linux-s390x-gnu@1.70.0':
    resolution: {integrity: sha512-Kwu/l/8GcYibCWA9m9N5pRXMIKVSsL/Ybg
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `frontend/pnpm-workspace.yaml`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/pnpm-workspace.yaml`
- Size bytes / Размер в байтах: `30`
- Included characters / Включено символов: `30`
- Truncated / Обрезано: `no`

```yaml
allowBuilds:
  vue-demi: true
```

### `frontend/src-tauri/.gitignore`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src-tauri/.gitignore`
- Size bytes / Размер в байтах: `86`
- Included characters / Включено символов: `86`
- Truncated / Обрезано: `no`

```text
# Generated by Cargo
# will have compiled files and executables
/target/
/gen/schemas
```

### `frontend/src-tauri/Cargo.lock`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src-tauri/Cargo.lock`
- Size bytes / Размер в байтах: `127309`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```text
# This file is automatically @generated by Cargo.
# It is not intended for manual editing.
version = 4

[[package]]
name = "adler2"
version = "2.0.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "320119579fcad9c21884f5c4861d16174d0e06250625266f50fe6898340abefa"

[[package]]
name = "ahash"
version = "0.7.8"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "891477e0c6a8957309ee5c45a6368af3ae14bb510732d2684ffa19af310920f9"
dependencies = [
 "getrandom 0.2.17",
 "once_cell",
 "version_check",
]

[[package]]
name = "aho-corasick"
version = "1.1.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "ddd31a130427c27518df266943a5308ed92d4b226cc639f5a8f1002816174301"
dependencies = [
 "memchr",
]

[[package]]
name = "alloc-no-stdlib"
version = "2.0.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "cc7bb162ec39d46ab1ca8c77bf72e890535becd1751bb45f64c597edb4c8c6b3"

[[package]]
name = "alloc-stdlib"
version = "0.2.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "94fb8275041c72129eb51b7d0322c29b8387a0386127718b096429201a5d6ece"
dependencies = [
 "alloc-no-stdlib",
]

[[package]]
name = "android_log-sys"
version = "0.3.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "84521a3cf562bc62942e294181d9eef17eb38ceb8c68677bc49f144e4c3d4f8d"

[[package]]
name = "android_logger"
version = "0.15.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "dbb4e440d04be07da1f1bf44fb4495ebd58669372fe0cffa6e48595ac5bd88a3"
dependencies = [
 "android_log-sys",
 "env_filter",
 "log",
]

[[package]]
name = "android_system_properties"
version = "0.1.5"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "819e7219dbd41043ac279b19830f2efc897156490d7fd6ea916720117ee66311"
dependencies = [
 "libc",
]

[[package]]
name = "anyhow"
version = "1.0.102"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "7f202df86484c868dbad7eaa557ef785d5c66295e41b460ef922eca0723b842c"

[[package]]
name = "app"
version = "0.1.0"
dependencies = [
 "log",
 "serde",
 "serde_json",
 "tauri",
 "tauri-build",
 "tauri-plugin-log",
 "tauri-plugin-shell",
 "ureq",
]

[[package]]
name = "arrayvec"
version = "0.7.6"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "7c02d123df017efcdfbd739ef81735b36c5ba83ec3c59c80a9d7ecc718f92e50"

[[package]]
name = "atk"
version = "0.18.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "241b621213072e993be4f6f3a9e4b45f65b7e6faad43001be957184b7bb1824b"
dependencies = [
 "atk-sys",
 "glib",
 "libc",
]

[[package]]
name = "atk-sys"
version = "0.18.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "c5e48b684b0ca77d2bbadeef17424c2ea3c897d44d566a1617e7e8f30614d086"
dependencies = [
 "glib-sys",
 "gobject-sys",
 "libc",
 "system-deps",
]

[[package]]
name = "atomic-waker"
version = "1.1.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "1505bd5d3d116872e7271a6d4e16d81d0c8570876c8de68093a09ac269d8aac0"

[[package]]
name = "autocfg"
version = "1.5.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "f2032f911046de80f0a198e0901378627c33f59ea0ac00e363d481118bd70a53"

[[package]]
name = "base64"
version = "0.21.7"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "9d297deb1925b89f2ccc13d7635fa0714f12c87adce1c75356b39ca9b7178567"

[[package]]
name = "base64"
version = "0.22.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "72b3254f16251a8381aa12e40e3c4d2f0199f8c6508fbecb9d91f575e0fbb8c6"

[[package]]
name = "bit-set"
version = "0.8.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "08807e080ed7f9d5433fa9b275196cfc35414f66a0c79d864dc51a0d825231a3"
dependencies = [
 "bit-vec",
]

[[package]]
name = "bit-vec"
version = "0.8.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "5e764a1d40d510daf35e07be9eb06e75770908c27d411ee6c92109c9840eaaf7"

[[package]]
name = "bitflags"
version = "1.3.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "bef38d45163c2f1dde094a7dfd33ccf595c92905c8f8f4fdc18d06fb1037718a"

[[package]]
name = "bitflags"
version = "2.12.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "84d7ced0ae9557296835c32bf1b1e02b44c746701f898460fb000d7eaa84f00a"
dependencies = [
 "serde_core",
]

[[package]]
name = "bitvec"
version = "1.0.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "1bc2832c24239b0141d5674bb9174f9d68a8b5b3f2753311927c172ca46f7e9c"
dependencies = [
 "funty",
 "radium",
 "tap",
 "wyz",
]

[[package]]
name = "block-buffer"
version = "0.10.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "3078c7629b62d3f0439517fa394996acacc5cbc91c5a20d8c658e77abd503a71"
dependencies = [
 "generic-array",
]

[[package]]
name = "block2"
version = "0.6.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "cdeb9d870516001442e364c5220d3574d2da8dc765554b4a617230d33fa58ef5"
dependencies = [
 "objc2",
]

[[package]]
name = "borsh"
version = "1.6.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "cfd1e3f8955a5d7de9fab72fc8373fade9fb8a703968cb200ae3dc6cf08e185a"
dependencies = [
 "borsh-derive",
 "bytes",
 "cfg_aliases",
]

[[package]]
name = "borsh-derive"
version = "1.6.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "bfcfdc083699101d5a7965e49925975f2f55060f94f9a05e7187be95d530ca59"
dependencies = [
 "once_cell",
 "proc-macro-crate 3.5.0",
 "proc-macro2",
 "quote",
 "syn 2.0.117",
]

[[package]]
name = "brotli"
version = "8.0.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "8119e4516436f5708bbc474a9d395bf12f1b5395e93a92a56e647ac3388c8610"
dependencies = [
 "alloc-no-stdlib",
 "alloc-stdlib",
 "brotli-decompressor",
]

[[package]]
name = "brotli-decompressor"
version = "5.0.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "5962523e1b92ce1b5e793d9169b9943eece10d39f62550bc04bb605d75b94924"
dependencies = [
 "alloc-no-stdlib",
 "alloc-stdlib",
]

[[package]]
name = "bs58"
version = "0.5.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "bf88ba1141d185c399bee5288d850d63b8369520c1eafc32a0430b5b6c287bf4"
dependencies = [
 "tinyvec",
]

[[package]]
name = "bumpalo"
version = "3.20.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "72f5acc6cb2ba439de613abc23857ec3d78374d8ed5ac84e9d11336e87da8649"

[[package]]
name = "byte-unit"
version = "5.2.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "8c6d47a4e2961fb8721bcfc54feae6455f2f64e7054f9bc67e875f0e77f4c58d"
dependencies = [
 "rust_decimal",
 "schemars 1.2.1",
 "serde",
 "utf8-width",
]

[[package]]
name = "bytecheck"
version = "0.6.12"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "23cdc57ce23ac53c931e88a43d06d070a6fd142f2617be5855eb75efc9beb1c2"
dependencies = [
 "bytecheck_derive",
 "ptr_meta",
 "simdutf8",
]

[[package]]
name = "bytecheck_derive"
version = "0.6.12"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "3db406d29fbcd95542e92559bed4d8ad92636d1ca8b3b72ede10b4bcc010e659"
dependencies = [
 "proc-macro2",
 "quote",
 "syn 1.0.109",
]

[[package]]
name = "bytemuck"
version = "1.25.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "c8efb64bd706a16a1bdde310ae86b351e4d21550d98d056f22f8a7f7a2183fec"

[[package]]
name = "byteorder"
version = "1.5.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "1fd0f2584146f6f2ef48085050886acf353beff7305ebd1ae69500e27c67f64b"

[[package]]
name = "bytes"
version = "1.11.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "1e748733b7cbc798e1434b6ac524f0c1ff2ab456fe201501e6497c8417a4fc33"
dependencies = [
 "serde",
]

[[package]]
name = "cairo-rs"
version = "0.18.5"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "8ca26ef0159422fb77631dc9d17b102f253b876fe1586b03b803e63a309b4ee2"
dependencies = [
 "bitflags 2.12.1",
 "cairo-sys-rs",
 "glib",
 "libc",
 "once_cell",
 "thiserror 1.0.69",
]

[[package]]
name = "cairo-sys-rs"
version = "0.18.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "685c9fa8e590b8b3d678873528d83411db17242a73fccaed827770ea0fedda51"
dependencies = [
 "glib-sys",
 "libc",
 "system-deps",
]

[[package]]
name = "camino"
version = "1.2.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "e629a66d692cb9ff1a1c664e41771b3dcaf961985a9774c0eb0bd1b51cf60a48"
dependencies = [
 "serde_core",
]

[[package]]
name = "cargo-platform"
version = "0.1.9"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "e35af189006b9c0f00a064685c727031e3ed2d8020f7ba284d78cc2671bd36ea"
dependencies = [
 "serde",
]

[[package]]
name = "cargo_metadata"
version = "0.19.2"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "dd5eb614ed4c27c5d706420e4320fbe3216ab31fa1c33cd8246ac36dae4479ba"
dependencies = [
 "camino",
 "cargo-platform",
 "semver",
 "serde",
 "serde_json",
 "thiserror 2.0.18",
]

[[package]]
name = "cargo_toml"
version = "0.22.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "374b7c592d9c00c1f4972ea58390ac6b18cbb6ab79011f3bdc90a0b82ca06b77"
dependencies = [
 "serde",
 "toml 0.9.12+spec-1.1.0",
]

[[package]]
name = "cc"
version = "1.2.63"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "556e016178bb5662a08681bbe0f00f8e17631781a4dfc8c45e466e4b185ec27f"
dependencies = [
 "find-msvc-tools",
 "shlex",
]

[[package]]
name = "cesu8"
version = "1.1.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "6d43a04d8753f35258c91f8ec639f792891f748a1edbd759cf1dcea3382ad83c"

[[package]]
name = "cfb"
version = "0.7.3"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "d38f2da7a0a2c4ccf0065be06397cc26a81f4e528be095826eee9d4adbb8c60f"
dependencies = [
 "byteorder",
 "fnv",
 "uuid",
]

[[package]]
name = "cfg-expr"
version = "0.15.8"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "d067ad48b8650848b989a59a86c6c36a995d02d2bf778d45c3c5d57bc2718f02"
dependencies = [
 "smallvec",
 "target-lexicon",
]

[[package]]
name = "cfg-if"
version = "1.0.4"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "9330f8b2ff13f34540b44e946ef35111825727b38d33286ef986142615121801"

[[package]]
name = "cfg_aliases"
version = "0.2.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "613afe47fcd5fac7ccf1db93babcb082c5994d996f20b8b159f2ad1658eb5724"

[[package]]
name = "chrono"
version = "0.4.45"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "1aa79e62e7697b8e29b513a68abacf485adcd1fe8284a4316c5ae868e6633327"
dependencies = [
 "iana-time-zone",
 "num-traits",
 "serde",
 "windows-link 0.2.1",
]

[[package]]
name = "combine"
version = "4.6.7"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "ba5a308b75df32fe02788e748662718f03fde005016435c444eea572398219fd"
dependencies = [
 "bytes",
 "memchr",
]

[[package]]
name = "cookie"
version = "0.18.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "4ddef33a339a91ea89fb53151bd0a4689cfce27055c291dfa69945475d22c747"
dependencies = [
 "percent-encoding",
 "time",
 "version_check",
]

[[package]]
name = "cookie_store"
version = "0.22.1"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "15b2c103cf610ec6cae3da84a766285b42fd16aad564758459e6ecf128c75206"
dependencies = [
 "coo
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `frontend/src-tauri/Cargo.toml`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src-tauri/Cargo.toml`
- Size bytes / Размер в байтах: `683`
- Included characters / Включено символов: `683`
- Truncated / Обрезано: `no`

```toml
[package]
name = "app"
version = "0.1.0"
description = "Макошь desktop shell"
authors = ["Макошь"]
license = ""
repository = ""
edition = "2024"
rust-version = "1.89"

# See more keys and their definitions at https://doc.rust-lang.org/cargo/reference/manifest.html

[lib]
name = "app_lib"
crate-type = ["staticlib", "cdylib", "rlib"]

[build-dependencies]
tauri-build = { version = "2.6.2", features = [] }

[dependencies]
serde_json = "1.0"
serde = { version = "1.0", features = ["derive"] }
log = "0.4"
tauri = { version = "2.11.2", features = [] }
tauri-plugin-log = "2"
tauri-plugin-shell = "2"
ureq = { version = "3.3.0", default-features = false, features = ["json"] }
```

### `frontend/src-tauri/capabilities/default.json`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src-tauri/capabilities/default.json`
- Size bytes / Размер в байтах: `524`
- Included characters / Включено символов: `524`
- Truncated / Обрезано: `no`

```json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "default",
  "description": "enables the default permissions",
  "windows": [
    "main"
  ],
  "permissions": [
    "core:default",
    "allow-open-whatsapp-web-companion",
    "allow-whatsapp-web-companion-manifest",
    "allow-open-yandex-telemost-companion",
    "allow-yandex-telemost-companion-manifest",
    "allow-yandex-telemost-prepare-audio-device",
    "allow-yandex-telemost-recording-start",
    "allow-yandex-telemost-recording-stop"
  ]
}
```

### `frontend/src-tauri/capabilities/whatsapp-companion-relay.json`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src-tauri/capabilities/whatsapp-companion-relay.json`
- Size bytes / Размер в байтах: `438`
- Included characters / Включено символов: `438`
- Truncated / Обрезано: `no`

```json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "whatsapp-companion-relay",
  "description": "Allows owner-visible WhatsApp Web companion windows to invoke only the sanitized relay preflight command.",
  "local": false,
  "remote": {
    "urls": [
      "https://web.whatsapp.com"
    ]
  },
  "windows": [
    "whatsapp-companion-*"
  ],
  "permissions": [
    "allow-whatsapp-web-companion-relay-observation"
  ]
}
```

### `frontend/src-tauri/capabilities/yandex-telemost-companion-relay.json`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src-tauri/capabilities/yandex-telemost-companion-relay.json`
- Size bytes / Размер в байтах: `483`
- Included characters / Включено символов: `483`
- Truncated / Обрезано: `no`

```json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "yandex-telemost-companion-relay",
  "description": "Allows owner-visible Yandex Telemost WebView windows to append sanitized active-speaker timeline hints only.",
  "local": false,
  "remote": {
    "urls": [
      "https://telemost.yandex.ru",
      "https://telemost.yandex.com"
    ]
  },
  "windows": [
    "yandex-telemost-*"
  ],
  "permissions": [
    "allow-yandex-telemost-speaker-timeline-append"
  ]
}
```

### `frontend/src-tauri/permissions/autogenerated/open_whatsapp_web_companion.toml`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src-tauri/permissions/autogenerated/open_whatsapp_web_companion.toml`
- Size bytes / Размер в байтах: `462`
- Included characters / Включено символов: `462`
- Truncated / Обрезано: `no`

```toml
# Automatically generated - DO NOT EDIT!

[[permission]]
identifier = "allow-open-whatsapp-web-companion"
description = "Enables the open_whatsapp_web_companion command without any pre-configured scope."
commands.allow = ["open_whatsapp_web_companion"]

[[permission]]
identifier = "deny-open-whatsapp-web-companion"
description = "Denies the open_whatsapp_web_companion command without any pre-configured scope."
commands.deny = ["open_whatsapp_web_companion"]
```

### `frontend/src-tauri/permissions/autogenerated/open_yandex_telemost_companion.toml`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src-tauri/permissions/autogenerated/open_yandex_telemost_companion.toml`
- Size bytes / Размер в байтах: `480`
- Included characters / Включено символов: `480`
- Truncated / Обрезано: `no`

```toml
# Automatically generated - DO NOT EDIT!

[[permission]]
identifier = "allow-open-yandex-telemost-companion"
description = "Enables the open_yandex_telemost_companion command without any pre-configured scope."
commands.allow = ["open_yandex_telemost_companion"]

[[permission]]
identifier = "deny-open-yandex-telemost-companion"
description = "Denies the open_yandex_telemost_companion command without any pre-configured scope."
commands.deny = ["open_yandex_telemost_companion"]
```

### `frontend/src-tauri/permissions/autogenerated/whatsapp_web_companion_manifest.toml`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src-tauri/permissions/autogenerated/whatsapp_web_companion_manifest.toml`
- Size bytes / Размер в байтах: `486`
- Included characters / Включено символов: `486`
- Truncated / Обрезано: `no`

```toml
# Automatically generated - DO NOT EDIT!

[[permission]]
identifier = "allow-whatsapp-web-companion-manifest"
description = "Enables the whatsapp_web_companion_manifest command without any pre-configured scope."
commands.allow = ["whatsapp_web_companion_manifest"]

[[permission]]
identifier = "deny-whatsapp-web-companion-manifest"
description = "Denies the whatsapp_web_companion_manifest command without any pre-configured scope."
commands.deny = ["whatsapp_web_companion_manifest"]
```

### `frontend/src-tauri/permissions/autogenerated/whatsapp_web_companion_relay_observation.toml`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src-tauri/permissions/autogenerated/whatsapp_web_companion_relay_observation.toml`
- Size bytes / Размер в байтах: `540`
- Included characters / Включено символов: `540`
- Truncated / Обрезано: `no`

```toml
# Automatically generated - DO NOT EDIT!

[[permission]]
identifier = "allow-whatsapp-web-companion-relay-observation"
description = "Enables the whatsapp_web_companion_relay_observation command without any pre-configured scope."
commands.allow = ["whatsapp_web_companion_relay_observation"]

[[permission]]
identifier = "deny-whatsapp-web-companion-relay-observation"
description = "Denies the whatsapp_web_companion_relay_observation command without any pre-configured scope."
commands.deny = ["whatsapp_web_companion_relay_observation"]
```

### `frontend/src-tauri/permissions/autogenerated/yandex_telemost_companion_manifest.toml`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src-tauri/permissions/autogenerated/yandex_telemost_companion_manifest.toml`
- Size bytes / Размер в байтах: `504`
- Included characters / Включено символов: `504`
- Truncated / Обрезано: `no`

```toml
# Automatically generated - DO NOT EDIT!

[[permission]]
identifier = "allow-yandex-telemost-companion-manifest"
description = "Enables the yandex_telemost_companion_manifest command without any pre-configured scope."
commands.allow = ["yandex_telemost_companion_manifest"]

[[permission]]
identifier = "deny-yandex-telemost-companion-manifest"
description = "Denies the yandex_telemost_companion_manifest command without any pre-configured scope."
commands.deny = ["yandex_telemost_companion_manifest"]
```

### `frontend/src-tauri/permissions/autogenerated/yandex_telemost_prepare_audio_device.toml`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src-tauri/permissions/autogenerated/yandex_telemost_prepare_audio_device.toml`
- Size bytes / Размер в байтах: `516`
- Included characters / Включено символов: `516`
- Truncated / Обрезано: `no`

```toml
# Automatically generated - DO NOT EDIT!

[[permission]]
identifier = "allow-yandex-telemost-prepare-audio-device"
description = "Enables the yandex_telemost_prepare_audio_device command without any pre-configured scope."
commands.allow = ["yandex_telemost_prepare_audio_device"]

[[permission]]
identifier = "deny-yandex-telemost-prepare-audio-device"
description = "Denies the yandex_telemost_prepare_audio_device command without any pre-configured scope."
commands.deny = ["yandex_telemost_prepare_audio_device"]
```

### `frontend/src-tauri/permissions/autogenerated/yandex_telemost_recording_start.toml`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src-tauri/permissions/autogenerated/yandex_telemost_recording_start.toml`
- Size bytes / Размер в байтах: `486`
- Included characters / Включено символов: `486`
- Truncated / Обрезано: `no`

```toml
# Automatically generated - DO NOT EDIT!

[[permission]]
identifier = "allow-yandex-telemost-recording-start"
description = "Enables the yandex_telemost_recording_start command without any pre-configured scope."
commands.allow = ["yandex_telemost_recording_start"]

[[permission]]
identifier = "deny-yandex-telemost-recording-start"
description = "Denies the yandex_telemost_recording_start command without any pre-configured scope."
commands.deny = ["yandex_telemost_recording_start"]
```

### `frontend/src-tauri/permissions/autogenerated/yandex_telemost_recording_stop.toml`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src-tauri/permissions/autogenerated/yandex_telemost_recording_stop.toml`
- Size bytes / Размер в байтах: `480`
- Included characters / Включено символов: `480`
- Truncated / Обрезано: `no`

```toml
# Automatically generated - DO NOT EDIT!

[[permission]]
identifier = "allow-yandex-telemost-recording-stop"
description = "Enables the yandex_telemost_recording_stop command without any pre-configured scope."
commands.allow = ["yandex_telemost_recording_stop"]

[[permission]]
identifier = "deny-yandex-telemost-recording-stop"
description = "Denies the yandex_telemost_recording_stop command without any pre-configured scope."
commands.deny = ["yandex_telemost_recording_stop"]
```

### `frontend/src-tauri/permissions/autogenerated/yandex_telemost_speaker_timeline_append.toml`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src-tauri/permissions/autogenerated/yandex_telemost_speaker_timeline_append.toml`
- Size bytes / Размер в байтах: `534`
- Included characters / Включено символов: `534`
- Truncated / Обрезано: `no`

```toml
# Automatically generated - DO NOT EDIT!

[[permission]]
identifier = "allow-yandex-telemost-speaker-timeline-append"
description = "Enables the yandex_telemost_speaker_timeline_append command without any pre-configured scope."
commands.allow = ["yandex_telemost_speaker_timeline_append"]

[[permission]]
identifier = "deny-yandex-telemost-speaker-timeline-append"
description = "Denies the yandex_telemost_speaker_timeline_append command without any pre-configured scope."
commands.deny = ["yandex_telemost_speaker_timeline_append"]
```

### `frontend/src-tauri/tauri.conf.json`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src-tauri/tauri.conf.json`
- Size bytes / Размер в байтах: `866`
- Included characters / Включено символов: `866`
- Truncated / Обрезано: `no`

```json
{
  "$schema": "../node_modules/@tauri-apps/cli/config.schema.json",
  "productName": "Макошь",
  "version": "0.1.0",
  "identifier": "dev.makosh.desktop",
  "build": {
    "frontendDist": "../dist",
    "devUrl": "http://127.0.0.1:5173"
  },
  "app": {
    "windows": [
      {
        "title": "Макошь",
        "width": 1280,
        "height": 800,
        "resizable": true,
        "fullscreen": false
      }
    ],
    "security": {
      "csp": null
    }
  },
  "bundle": {
    "active": true,
    "targets": "all",
    "externalBin": [
      "binaries/makosh-backend"
    ],
    "resources": {
      "resources/google-oauth/": "google-oauth",
      "resources/tdlib/": "tdlib"
    },
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/128x128@2x.png",
      "icons/icon.icns",
      "icons/icon.ico"
    ]
  }
}
```

### `frontend/src/platform/i18n/en.json`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/platform/i18n/en.json`
- Size bytes / Размер в байтах: `40328`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```json
{
	"Home": "Home",
	"Communications": "Communications",
	"Timeline": "Timeline",
	"Persons": "Persons",
	"Projects": "Projects",
	"Tasks": "Tasks",
	"Calendar": "Calendar",
	"Documents": "Documents",
	"Notes": "Notes",
	"Knowledge Graph": "Knowledge Graph",
	"AI Agents": "AI Agents",
	"Settings": "Settings",
	"Widget grid size": "Widget grid size",
	"Widget layout controls": "Widget layout controls",
	"Widget layout saved": "Widget layout saved",
	"Configure widget": "Configure widget",
	"Widget settings": "Widget settings",
	"Close widget settings": "Close widget settings",
	"Widget ID": "Widget ID",
	"Zone": "Zone",
	"Add widget": "Add widget",
	"Cancel": "Cancel",
	"Reset": "Reset",
	"Save": "Save",
	"Saving": "Saving",
	"Cols": "Cols",
	"Rows": "Rows",
	"Widget columns": "Widget columns",
	"Widget rows": "Widget rows",
	"Decrease columns": "Decrease columns",
	"Increase columns": "Increase columns",
	"Decrease rows": "Decrease rows",
	"Increase rows": "Increase rows",
	"Move widget up": "Move widget up",
	"Move widget down": "Move widget down",
	"Hide widget": "Hide widget",
	"Reset widget size": "Reset widget size",
	"Unified": "Unified",
	"Inbox": "Inbox",
	"Waiting": "Waiting",
	"Needs Reply": "Needs Reply",
	"Mentions": "Mentions",
	"Mail": "Mail",
	"Calls": "Calls",
	"Meetings": "Meetings",
	"Telegram": "Telegram",
	"WhatsApp": "WhatsApp",
	"Header": "Header",
	"Filters": "Filters",
	"List": "List",
	"Detail": "Detail",
	"Rail": "Rail",
	"Hero": "Hero",
	"Metrics": "Metrics",
	"Main": "Main",
	"Bottom": "Bottom",
	"Metadata": "Metadata",
	"Tabs": "Tabs",
	"Toolbar": "Toolbar",
	"Canvas": "Canvas",
	"Inspector": "Inspector",
	"Home Metrics": "Home Metrics",
	"Focus Score": "Focus Score",
	"What's New": "What's New",
	"Today's Priorities": "Today's Priorities",
	"Upcoming": "Upcoming",
	"Yesterday": "Yesterday",
	"People You Talked To": "People You Talked To",
	"System Status": "System Status",
	"Active Projects": "Active Projects",
	"Conversation List": "Conversation List",
	"Message Detail": "Message Detail",
	"Sender Profile": "Sender Profile",
	"Summary": "Summary",
	"Message Metadata": "Message Metadata",
	"Related Projects": "Related Projects",
	"Active Tasks": "Active Tasks",
	"Ask AI": "Ask AI",
	"Timeline Stream": "Timeline Stream",
	"Timeline Filters": "Timeline Filters",
	"Period Summary": "Period Summary",
	"Selected Event Context": "Selected Event Context",
	"Persons List": "Persons List",
	"Person Hero": "Person Hero",
	"Person Information": "Person Information",
	"About": "About",
	"Relationship Strength": "Relationship Strength",
	"Recent Interactions": "Recent Interactions",
	"AI Summary": "AI Summary",
	"Person Identity Review": "Person Identity Review",
	"Related Documents": "Related Documents",
	"Recent Notes": "Recent Notes",
	"Project Hero": "Project Hero",
	"Metadata Strip": "Metadata Strip",
	"Project Switcher": "Project Switcher",
	"Section Tabs": "Section Tabs",
	"Project Summary": "Project Summary",
	"Project Timeline": "Project Timeline",
	"Recent Communications": "Recent Communications",
	"Top Documents": "Top Documents",
	"Source Evidence": "Source Evidence",
	"Open Promises": "Open Promises",
	"Project Health": "Project Health",
	"Key People": "Key People",
	"Task Metrics": "Task Metrics",
	"Candidate Review Queue": "Candidate Review Queue",
	"AI Refresh Status": "AI Refresh Status",
	"Task Context": "Task Context",
	"Deadlines And Priority": "Deadlines And Priority",
	"Calendar Toolbar": "Calendar Toolbar",
	"Week Grid": "Week Grid",
	"Event Blocks": "Event Blocks",
	"Source Status": "Source Status",
	"Source Cards": "Source Cards",
	"Document Navigation": "Document Navigation",
	"Documents List": "Documents List",
	"Document Detail": "Document Detail",
	"Processing Jobs": "Processing Jobs",
	"Failed Job Retry Status": "Failed Job Retry Status",
	"Related Context": "Related Context",
	"Notes List": "Notes List",
	"Note Detail": "Note Detail",
	"Note Metadata": "Note Metadata",
	"Source Filters": "Source Filters",
	"Related Projects And Documents": "Related Projects And Documents",
	"Graph Toolbar": "Graph Toolbar",
	"Graph Canvas": "Graph Canvas",
	"Node Inspector": "Node Inspector",
	"Connections": "Connections",
	"Graph Summary": "Graph Summary",
	"Search Results": "Search Results",
	"Evidence": "Evidence",
	"Telegram Chats": "Telegram Chats",
	"Message Thread": "Message Thread",
	"Account Status": "Account Status",
	"Sync Controls": "Sync Controls",
	"Selected Chat Metadata": "Selected Chat Metadata",
	"Session Status": "Session Status",
	"Chat Message Surface": "Chat Message Surface",
	"Account Session Metadata": "Account Session Metadata",
	"Runtime Metrics": "Runtime Metrics",
	"Runtime Status": "Runtime Status",
	"Agent List": "Agent List",
	"Selected Agent Detail": "Selected Agent Detail",
	"Run History": "Run History",
	"Answer Form": "Answer Form",
	"Meeting Prep And Task Extraction": "Meeting Prep And Task Extraction",
	"Citations": "Citations",
	"Company List": "Company List",
	"Company Detail": "Company Detail",
	"Company Health": "Company Health",
	"Settings Metrics": "Settings Metrics",
	"Application Settings": "Application Settings",
	"All non-secret settings except database connectivity; secret-like keys are rejected.": "All non-secret settings except database connectivity; secret-like keys are rejected.",
	"Loading settings...": "Loading settings...",
	"No application settings are declared yet.": "No application settings are declared yet.",
	"Enabled": "Enabled",
	"Disabled": "Disabled",
	"Runtime Source": "Runtime Source",
	"Backend bind": "Backend bind",
	"Frontend API": "Frontend API",
	"AI configuration": "AI configuration",
	"Model routing": "Model routing",
	"Capability slots": "Capability slots",
	"Boundaries": "Boundaries",
	"PostgreSQL stores declared setting values": "PostgreSQL stores declared setting values",
	"AI providers, models and routes live in AI Control Center": "AI providers, models and routes live in AI Control Center",
	"Domain tables": "Domain tables",
	"Legacy ai.* settings are bootstrap fallback only": "Legacy ai.* settings are bootstrap fallback only",
	"Hidden": "Hidden",
	"Database URL stays outside the panel": "Database URL stays outside the panel",
	"Bootstrap": "Bootstrap",
	"API token and vault key stay outside DB": "API token and vault key stay outside DB",
	"Secret boundary": "Secret boundary",
	"Credentials stay in encrypted vault": "Credentials stay in encrypted vault",
	"No secret values": "No secret values",
	"Settings updates are audited": "Settings updates are audited",
	"No values in audit": "No values in audit",
	"Accounts List": "Accounts List",
	"Account Setup": "Account Setup",
	"Account Detail Status": "Account Detail Status",
	"Security And Runtime Status": "Security And Runtime Status",
	"Minimum window size is 800 x 600": "Minimum window size is 800 x 600",
	"Increase the Макошь window size to continue.": "Increase the Макошь window size to continue.",
	"Good evening, Alex": "Good evening, Alex",
	"Here's what's happening in your world today.": "Here's what's happening in your world today.",
	"All your conversations. All channels. One place.": "All your conversations. All channels. One place.",
	"Chronological activity across messages, tasks, documents and meetings.": "Chronological activity across messages, tasks, documents and meetings.",
	"642 persons": "642 persons",
	"Product Development": "Product Development",
	"All your tasks from connected trackers": "All your tasks from connected trackers",
	"All your events from connected calendars": "All your events from connected calendars",
	"All your documents from connected sources": "All your documents from connected sources",
	"All your notes from connected sources": "All your notes from connected sources",
	"Explore relationships across people, projects, documents, messages and tasks.": "Explore relationships across people, projects, documents, messages and tasks.",
	"Telegram messaging, policy automation and call intelligence.": "Telegram messaging, policy automation and call intelligence.",
	"WhatsApp companion sessions, fixture ingestion and live-runtime guardrails.": "WhatsApp companion sessions, fixture ingestion and live-runtime guardrails.",
	"Your intelligent assistants working across your data and tools": "Your intelligent assistants working across your data and tools",
	"All companies and organizations from your communications": "All companies and organizations from your communications",
	"Runtime settings and connected accounts.": "Runtime settings and connected accounts.",
	"Макошь": "Макошь",
	"Telegram Client": "Telegram Client",
	"WhatsApp Web": "WhatsApp Web",
	"Companies": "Companies",
	"Chronological activity across connected sources.": "Chronological activity across connected sources.",
	"Search anything...": "Search anything...",
	"Search in communications...": "Search in communications...",
	"Search timeline...": "Search timeline...",
	"Search persons, companies, emails...": "Search persons, companies, emails...",
	"Search projects, documents, people...": "Search projects, documents, people...",
	"Search tasks, projects, trackers, people...": "Search tasks, projects, trackers, people...",
	"Search events, meetings, persons...": "Search events, meetings, persons...",
	"Search documents, folders, content...": "Search documents, folders, content...",
	"Search notes, content, emails...": "Search notes, content, emails...",
	"Search anything in your knowledge graph...": "Search anything in your knowledge graph...",
	"Search Telegram chats, policies, calls...": "Search Telegram chats, policies, calls...",
	"Search WhatsApp sessions and messages...": "Search WhatsApp sessions and messages...",
	"Search agents, capabilities, tasks...": "Search agents, capabilities, tasks...",
	"Search companies, industries, locations...": "Search companies, industries, locations...",
	"Search settings and accounts...": "Search settings and accounts...",
	"Personal OS": "Personal OS",
	"Messages": "Messages",
	"Needs attention": "Needs attention",
	"Макошь Secure Vault": "Макошь Secure Vault",
	"Create Your Personal Secure Vault": "Create Your Personal Secure Vault",
	"Unlock Secure Vault": "Unlock Secure Vault",
	"Макошь needs the secure vault unlocked before it can save provider credentials on this device.": "Макошь needs the secure vault unlocked before it can save provider credentials on this device.",
	"Unlock Existing Vault": "Unlock Existing Vault",
	"Макошь Secure Vault is locked. Unlock the vault, then start Google mail connection again.": "Макошь Secure Vault is locked. Unlock the vault, then start Google mail connection again.",
	"Макошь Secure Vault is not initialized. Create the vault, then start Google mail connection again.": "Макошь Secure Vault is not initialized. Create the vault, then start Google mail connection again.",
	"Notifications": "Notifications",
	"No active notifications.": "No active notifications.",
	"Key changes and important updates": "Key changes and important updates",
	"Focus on what matters most": "Focus on what matters most",
	"Your schedule": "Your schedule",
	"View all": "View all",
	"Compose": "Compose",
	"To": "To",
	"CC": "CC",
	"Subject": "Subject",
	"Body": "Body",
	"Save Draft": "Save Draft",
	"Send": "Send",
	"Continue": "Continue",
	"Exit": "Exit",
	"Layout Settings": "Layout Settings",
	"Command Palette": "Command Palette",
	"Logout": "Logout",
	"Constructor Mode": "Constructor Mode",
	"Application": "Application",
	"Sidebar": "Sidebar",
	"Accounts": "Accounts",
	"Day": "Day",
	"Week": "Week",
	"Month": "Month",
	"Agenda": "Agenda",
	"Add Calendar": "Add Calendar",
	"Weekly Brief": "Weekly Brief",
	"Calendars": "Calendars",
	"Local Calendar": "Local Calendar",
	"Google Calendar": "Google Calendar",
	"Google OAuth client credentials are not configured. Add MAKOSH_GOOGLE_OAUTH_CLIENT_CONFIG_PATH or MAKOSH_GOOGLE_OAUTH_CLIENT_ID to docker/.env, the
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `frontend/src/platform/i18n/ru.json`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/platform/i18n/ru.json`
- Size bytes / Размер в байтах: `56199`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```json
{
	"Home": "Главная",
	"Communications": "Коммуникации",
	"Timeline": "Хронология",
	"Persons": "Люди",
	"Projects": "Проекты",
	"Tasks": "Задачи",
	"Calendar": "Календарь",
	"Documents": "Документы",
	"Notes": "Заметки",
	"Knowledge Graph": "Граф знаний",
	"AI Agents": "ИИ-агенты",
	"Settings": "Настройки",
	"Widget grid size": "Размер виджета",
	"Widget layout controls": "Управление layout виджетов",
	"Widget layout saved": "Layout виджетов сохранён",
	"Configure widget": "Настроить виджет",
	"Widget settings": "Настройки виджета",
	"Close widget settings": "Закрыть настройки виджета",
	"Widget ID": "ID виджета",
	"Zone": "Зона",
	"Add widget": "Добавить виджет",
	"Cancel": "Отмена",
	"Reset": "Сбросить",
	"Save": "Сохранить",
	"Saving": "Сохранение",
	"Cols": "Кол.",
	"Rows": "Стр.",
	"Widget columns": "Колонки виджета",
	"Widget rows": "Строки виджета",
	"Decrease columns": "Уменьшить колонки",
	"Increase columns": "Увеличить колонки",
	"Decrease rows": "Уменьшить строки",
	"Increase rows": "Увеличить строки",
	"Move widget up": "Переместить виджет вверх",
	"Move widget down": "Переместить виджет вниз",
	"Hide widget": "Скрыть виджет",
	"Reset widget size": "Сбросить размер виджета",
	"Unified": "Все",
	"Inbox": "Входящие",
	"Waiting": "Ожидание",
	"Needs Reply": "Нужен ответ",
	"Mentions": "Упоминания",
	"Mail": "Почта",
	"Calls": "Звонки",
	"Meetings": "Встречи",
	"Telegram": "Telegram",
	"WhatsApp": "WhatsApp",
	"Header": "Заголовок",
	"Filters": "Фильтры",
	"List": "Список",
	"Detail": "Детали",
	"Rail": "Боковая панель",
	"Hero": "Обзор",
	"Metrics": "Метрики",
	"Main": "Основное",
	"Bottom": "Нижняя панель",
	"Metadata": "Метаданные",
	"Tabs": "Вкладки",
	"Toolbar": "Панель инструментов",
	"Canvas": "Холст",
	"Inspector": "Инспектор",
	"Home Metrics": "Метрики главной",
	"Focus Score": "Фокус-метрика",
	"What's New": "Что нового",
	"Today's Priorities": "Приоритеты на сегодня",
	"Upcoming": "Предстоящее",
	"Yesterday": "Вчера",
	"People You Talked To": "С кем общались",
	"System Status": "Состояние системы",
	"Active Projects": "Активные проекты",
	"Conversation List": "Список бесед",
	"Message Detail": "Детали сообщения",
	"Sender Profile": "Профиль отправителя",
	"Summary": "Сводка",
	"Message Metadata": "Метаданные сообщения",
	"Related Projects": "Связанные проекты",
	"Active Tasks": "Активные задачи",
	"Ask AI": "Спросить ИИ",
	"Timeline Stream": "Поток хронологии",
	"Timeline Filters": "Фильтры хронологии",
	"Period Summary": "Сводка за период",
	"Selected Event Context": "Контекст события",
	"Persons List": "Список людей",
	"Person Hero": "Обзор персоны",
	"Person Information": "Информация о персоне",
	"About": "О персоне",
	"Relationship Strength": "Сила отношений",
	"Recent Interactions": "Недавние взаимодействия",
	"AI Summary": "ИИ-сводка",
	"Person Identity Review": "Проверка идентичности",
	"Related Documents": "Связанные документы",
	"Recent Notes": "Недавние заметки",
	"Project Hero": "Обзор проекта",
	"Metadata Strip": "Метаданные",
	"Project Switcher": "Переключение проектов",
	"Section Tabs": "Вкладки разделов",
	"Project Summary": "Сводка проекта",
	"Project Timeline": "Хронология проекта",
	"Recent Communications": "Недавние коммуникации",
	"Top Documents": "Основные документы",
	"Source Evidence": "Источники",
	"Open Promises": "Открытые обещания",
	"Project Health": "Состояние проекта",
	"Key People": "Ключевые люди",
	"Task Metrics": "Метрики задач",
	"Candidate Review Queue": "Очередь проверки кандидатов",
	"AI Refresh Status": "Статус ИИ-обновления",
	"Task Context": "Контекст задачи",
	"Deadlines And Priority": "Сроки и приоритет",
	"Calendar Toolbar": "Панель календаря",
	"Week Grid": "Сетка недели",
	"Event Blocks": "Блоки событий",
	"Source Status": "Статус источника",
	"Source Cards": "Карточки источников",
	"Document Navigation": "Навигация по документам",
	"Documents List": "Список документов",
	"Document Detail": "Детали документа",
	"Processing Jobs": "Задачи обработки",
	"Failed Job Retry Status": "Статус повтора ошибок",
	"Related Context": "Связанный контекст",
	"Notes List": "Список заметок",
	"Note Detail": "Детали заметки",
	"Note Metadata": "Метаданные заметки",
	"Source Filters": "Фильтры источников",
	"Related Projects And Documents": "Связанные проекты и документы",
	"Graph Toolbar": "Панель графа",
	"Graph Canvas": "Холст графа",
	"Node Inspector": "Инспектор узла",
	"Connections": "Связи",
	"Graph Summary": "Сводка графа",
	"Search Results": "Результаты поиска",
	"Evidence": "Доказательства",
	"Telegram Chats": "Чаты Telegram",
	"Message Thread": "Цепочка сообщений",
	"Account Status": "Статус аккаунта",
	"Sync Controls": "Управление синхронизацией",
	"Selected Chat Metadata": "Метаданные чата",
	"Session Status": "Статус сессии",
	"Chat Message Surface": "Поверхность сообщений",
	"Account Session Metadata": "Метаданные сессии",
	"Runtime Metrics": "Метрики исполнения",
	"Runtime Status": "Статус исполнения",
	"Agent List": "Список агентов",
	"Selected Agent Detail": "Детали агента",
	"Run History": "История запусков",
	"Answer Form": "Форма запроса",
	"Meeting Prep And Task Extraction": "Подготовка к встречам и извлечение задач",
	"Citations": "Цитаты",
	"Company List": "Список компаний",
	"Company Detail": "Детали компании",
	"Company Health": "Состояние компании",
	"Settings Metrics": "Метрики настроек",
	"Application Settings": "Настройки приложения",
	"All non-secret settings except database connectivity; secret-like keys are rejected.": "Все несекретные настройки, кроме подключения к базе; ключи, похожие на секреты, отклоняются.",
	"Loading settings...": "Загрузка настроек...",
	"No application settings are declared yet.": "Настройки приложения ещё не объявлены.",
	"Enabled": "Включено",
	"Disabled": "Отключено",
	"Saving": "Сохранение",
	"Runtime Source": "Источник runtime",
	"Backend bind": "Backend bind",
	"Frontend API": "Frontend API",
	"AI configuration": "AI-конфигурация",
	"Model routing": "Маршрутизация моделей",
	"Capability slots": "Capability slots",
	"Boundaries": "Границы",
	"PostgreSQL stores declared setting values": "PostgreSQL хранит значения объявленных настроек",
	"AI providers, models and routes live in AI Control Center": "AI-провайдеры, модели и маршруты живут в AI Control Center",
	"Domain tables": "Доменные таблицы",
	"Legacy ai.* settings are bootstrap fallback only": "Legacy ai.* настройки только bootstrap fallback",
	"Hidden": "Скрыто",
	"Database URL stays outside the panel": "Database URL остаётся вне панели",
	"Bootstrap": "Bootstrap",
	"API token and vault key stay outside DB": "API token и vault key остаются вне БД",
	"Secret boundary": "Граница секретов",
	"Credentials stay in encrypted vault": "Учётные данные остаются в encrypted vault",
	"No secret values": "Без секретных значений",
	"Settings updates are audited": "Изменения настроек аудируются",
	"No values in audit": "Без значений в audit",
	"Accounts List": "Список аккаунтов",
	"Account Setup": "Настройка аккаунта",
	"Account Detail Status": "Статус аккаунта",
	"Security And Runtime Status": "Безопасность и состояние",
	"Minimum window size is 800 x 600": "Минимальный размер окна: 800 × 600",
	"Increase the Макошь window size to continue.": "Увеличьте окно Макошь для продолжения.",
	"Good evening, Alex": "Добрый вечер, Алекс",
	"Here's what's happening in your world today.": "Главное за сегодня.",
	"All your conversations. All channels. One place.": "Все ваши разговоры. Все каналы. В одном месте.",
	"Chronological activity across messages, tasks, documents and meetings.": "Хронология: сообщения, задачи, документы и встречи.",
	"642 persons": "642 персоны",
	"Product Development": "Разработка продукта",
	"All your tasks from connected trackers": "Все задачи из подключённых трекеров",
	"All your events from connected calendars": "Все события из подключённых календарей",
	"All your documents from connected sources": "Все документы из подключённых источников",
	"All your notes from connected sources": "Все заметки из подключённых источников",
	"Explore relationships across people, projects, documents, messages and tasks.": "Исследуйте связи между людьми, проектами, документами.",
	"Telegram messaging, policy automation and call intelligence.": "Telegram: сообщения, политики, аналитика звонков.",
	"WhatsApp companion sessions, fixture ingestion and live-runtime guardrails.": "WhatsApp: сессии, фикстуры, ограничения.",
	"Your intelligent assistants working across your data and tools": "Ваши интеллектуальные ассистенты",
	"All companies and organizations from your communications": "Все компании и организации из коммуникаций",
	"Runtime settings and connected accounts.": "Настройки среды и подключённые аккаунты.",
	"Макошь": "Макошь",
	"Telegram Client": "Telegram",
	"WhatsApp Web": "WhatsApp Web",
	"Companies": "Компании",
	"Chronological activity across connected sources.": "Хронология по подключённым источникам.",
	"Search anything...": "Поиск...",
	"Search in communications...": "Поиск в коммуникациях...",
	"Search timeline...": "Поиск в хронологии...",
	"Search persons, companies, emails...": "Поиск людей, компаний, адресов...",
	"Search projects, documents, people...": "Поиск проектов, документов, людей...",
	"Search tasks, projects, trackers, people...": "Поиск задач, проектов, трекеров...",
	"Search events, meetings, persons...": "Поиск событий, встреч, персон...",
	"Search documents, folders, content...": "Поиск документов, папок, содержимого...",
	"Search notes, content, emails...": "Поиск заметок, писем...",
	"Search anything in your knowledge graph...": "Поиск по графу знаний...",
	"Search Telegram chats, policies, calls...": "Поиск чатов Telegram...",
	"Search WhatsApp sessions and messages...": "Поиск сессий WhatsApp...",
	"Search agents, capabilities, tasks...": "Поиск агентов, возможностей, задач...",
	"Search companies, industries, locations...": "Поиск компаний, отраслей, локаций...",
	"Search settings and accounts...": "Поиск настроек и аккаунтов...",
	"Personal OS": "Персональная ОС",
	"Messages": "Сообщения",
	"Needs attention": "Требуют внимания",
	"Макошь Secure Vault": "Защищённое хранилище Макошь",
	"Create Your Personal Secure Vault": "Создайте личное защищённое хранилище",
	"Unlock Secure Vault": "Разблокируйте защищённое хранилище",
	"Макошь needs the secure vault unlocked before it can save provider credentials on this device.": "Макошь нужно разблокированное защищённое хранилище, чтобы сохранять учётные данные провайдеров на этом устройстве.",
	"Unlock Existing Vault": "Разблокировать хранилище",
	"Макошь Secure Vault is locked. Unlock the vault, then start Google mail connection again.": "Защищённое хранилище Макошь заблокировано. Разблокируйте хранилище, затем снова начните подключение Google Mail.",
	"Макошь Secure Vault is not initialized. Create the vault, then start Google mail connection again.": "Защищённое хранилище Макошь ещё не создано. Создайте хранилище, затем снова начните подключение Google Mail.",
	"Notifications": "Уведомления",
	"No active notifications.": "Нет активных уведомлений.",
	"Key changes and important updates": "Ключевые изменения и обновления",
	"Focus on what matters most": "Сосредоточьтесь на главном",
	"Your schedule": "Ваше расписание",
	"View all": "Смотреть все",
	"Compose": "Написать",
	"To": "Кому",
	"CC": "Копия",
	"Subject": "Тема",
	"Body": "Текст",
	"Save Draft": "Сохранить черновик",
	"Send": "Отправить",
	"Continue": "Продолжить",
	"Exit": "Выйти",
	"Layout Settings": "Настройки макета",
	"Command Palette": "Командная палитра",
	"Logout": "Выход",
	"Constructor Mode": "Режим конструктора",
	"Application": "Приложение",
	"Sidebar": "Боковая панель",
	"Accounts": "Аккаунты",
	"Day": "День",
	"Week": "Неделя",
	"Month": "Месяц",
	"Agenda": "Повестка",
	"Add Calendar": "Добавить календарь",
	"Weekly Brief": "Недельный брифинг",
	"Calendars": "Календари",
	"Local Calendar": "Локальный календарь",
	"Google Calendar": "Google Календарь",
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `frontend/tsconfig.json`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/tsconfig.json`
- Size bytes / Размер в байтах: `538`
- Included characters / Включено символов: `538`
- Truncated / Обрезано: `no`

```json
{
	"compilerOptions": {
		"target": "ES2022",
		"module": "ESNext",
		"moduleResolution": "bundler",
		"strict": true,
		"jsx": "preserve",
		"resolveJsonModule": true,
		"isolatedModules": true,
		"esModuleInterop": true,
		"forceConsistentCasingInFileNames": true,
		"skipLibCheck": true,
		"sourceMap": true,
		"ignoreDeprecations": "6.0",
		"baseUrl": ".",
		"paths": {
			"@/*": ["src/*"]
		},
		"types": ["node"]
	},
	"include": ["src/**/*.ts", "src/**/*.vue", "src/**/*.d.ts"],
	"exclude": ["node_modules", "dist", "src-svelte"]
}
```
