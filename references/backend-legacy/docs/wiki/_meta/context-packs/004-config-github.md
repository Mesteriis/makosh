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

- Chunk ID / ID чанка: `004-config-github`
- Group / Группа: `.github`
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

### `.github/workflows/ci.yml`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/.github/workflows/ci.yml`
- Size bytes / Размер в байтах: `8511`
- Included characters / Включено символов: `8511`
- Truncated / Обрезано: `no`

```yaml
name: CI

on:
  pull_request:
  push:
    branches:
      - main

jobs:
  architecture:
    name: Architecture
    runs-on: ubuntu-latest
    timeout-minutes: 10

    steps:
      - name: Checkout
        uses: actions/checkout@v4

      - name: Install Node
        uses: actions/setup-node@v4
        with:
          node-version: '24'

      - name: Install Rust toolchain
        run: |
          rustup toolchain install 1.88 --profile minimal
          rustup default 1.88

      - name: Install cargo nextest
        uses: taiki-e/install-action@v2
        with:
          tool: cargo-nextest

      - name: Validate architecture boundaries
        run: make test-architecture

  backend-fmt:
    name: Backend fmt
    runs-on: ubuntu-latest
    timeout-minutes: 15
    env:
      CARGO_TARGET_ROOT: /tmp/makosh-cargo-target-${{ github.job }}

    steps:
      - name: Checkout
        uses: actions/checkout@v4

      - name: Install Rust toolchain
        run: |
          rustup toolchain install 1.88 --profile minimal --component rustfmt
          rustup default 1.88

      - name: Validate backend formatting
        run: make backend-fmt-check

  backend-clippy:
    name: Backend clippy
    runs-on: ubuntu-latest
    timeout-minutes: 25
    env:
      CARGO_TARGET_ROOT: /tmp/makosh-cargo-target-${{ github.job }}

    steps:
      - name: Checkout
        uses: actions/checkout@v4

      - name: Install Rust toolchain
        run: |
          rustup toolchain install 1.88 --profile minimal --component clippy
          rustup default 1.88

      - name: Cache Cargo registry and git sources
        uses: actions/cache@v4
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
          key: cargo-${{ runner.os }}-${{ hashFiles('**/Cargo.lock') }}

      - name: Validate backend clippy
        run: make backend-clippy

  backend-unit:
    name: Backend unit
    runs-on: ubuntu-latest
    timeout-minutes: 20
    env:
      CARGO_TARGET_ROOT: /tmp/makosh-cargo-target-${{ github.job }}

    steps:
      - name: Checkout
        uses: actions/checkout@v4

      - name: Install Rust toolchain
        run: |
          rustup toolchain install 1.88 --profile minimal
          rustup default 1.88

      - name: Install Node
        uses: actions/setup-node@v4
        with:
          node-version: '24'

      - name: Install cargo nextest
        uses: taiki-e/install-action@v2
        with:
          tool: cargo-nextest

      - name: Cache Cargo registry and git sources
        uses: actions/cache@v4
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
          key: cargo-${{ runner.os }}-${{ hashFiles('**/Cargo.lock') }}

      - name: Run backend unit tests
        run: make test-unit

      - name: Upload backend unit reports
        if: always()
        uses: actions/upload-artifact@v4
        with:
          name: backend-unit-reports
          path: |
            reports/test-performance/unit.json
            reports/test-performance/unit.md
            target/nextest/default/junit.xml

  backend-snapshot:
    name: Backend snapshots
    runs-on: ubuntu-latest
    timeout-minutes: 15
    env:
      CARGO_TARGET_ROOT: /tmp/makosh-cargo-target-${{ github.job }}

    steps:
      - name: Checkout
        uses: actions/checkout@v4

      - name: Install Rust toolchain
        run: |
          rustup toolchain install 1.88 --profile minimal
          rustup default 1.88

      - name: Install Node
        uses: actions/setup-node@v4
        with:
          node-version: '24'

      - name: Install cargo nextest
        uses: taiki-e/install-action@v2
        with:
          tool: cargo-nextest

      - name: Run snapshot tests
        run: make test-snapshot

  backend-integration:
    name: Backend integration
    if: github.event_name == 'push' && github.ref == 'refs/heads/main'
    runs-on: ubuntu-latest
    timeout-minutes: 45
    env:
      CARGO_TARGET_ROOT: /tmp/makosh-cargo-target-${{ github.job }}

    steps:
      - name: Checkout
        uses: actions/checkout@v4

      - name: Install Rust toolchain
        run: |
          rustup toolchain install 1.88 --profile minimal
          rustup default 1.88

      - name: Check Docker Compose
        run: docker compose version

      - name: Install cargo nextest
        uses: taiki-e/install-action@v2
        with:
          tool: cargo-nextest

      - name: Run backend integration tests
        run: make test-integration

      - name: Upload backend integration reports
        if: always()
        uses: actions/upload-artifact@v4
        with:
          name: backend-integration-reports
          path: |
            reports/test-performance/integration.json
            reports/test-performance/integration.md
            target/nextest/integration/junit.xml

  coverage:
    name: Coverage
    if: github.event_name == 'push' && github.ref == 'refs/heads/main'
    runs-on: ubuntu-latest
    timeout-minutes: 60
    env:
      CARGO_TARGET_ROOT: /tmp/makosh-cargo-target-${{ github.job }}

    steps:
      - name: Checkout
        uses: actions/checkout@v4

      - name: Install Rust toolchain
        run: |
          rustup toolchain install 1.88 --profile minimal --component llvm-tools-preview
          rustup default 1.88

      - name: Check Docker Compose
        run: docker compose version

      - name: Install coverage toolchain
        uses: taiki-e/install-action@v2
        with:
          tool: cargo-nextest,cargo-llvm-cov

      - name: Generate LCOV coverage
        run: make coverage-ci

      - name: Upload coverage artifacts
        if: always()
        uses: actions/upload-artifact@v4
        with:
          name: backend-coverage
          path: |
            target/coverage/lcov.info
            target/nextest/ci/junit.xml

  security:
    name: Security
    if: github.event_name == 'push' && github.ref == 'refs/heads/main'
    runs-on: ubuntu-latest
    timeout-minutes: 20

    steps:
      - name: Checkout
        uses: actions/checkout@v4

      - name: Install Rust toolchain
        run: |
          rustup toolchain install 1.88 --profile minimal
          rustup default 1.88

      - name: Install security tools
        uses: taiki-e/install-action@v2
        with:
          tool: cargo-audit,cargo-deny

      - name: Run security checks
        run: make security

  frontend-lint:
    name: Frontend lint
    runs-on: ubuntu-latest
    timeout-minutes: 15

    steps:
      - name: Checkout
        uses: actions/checkout@v4

      - name: Install Node
        uses: actions/setup-node@v4
        with:
          node-version: '24'

      - name: Install pnpm from frontend package
        run: |
          corepack enable
          PNPM_VERSION="$(node -p "require('./frontend/package.json').packageManager.split('@')[1]")"
          corepack prepare "pnpm@${PNPM_VERSION}" --activate

      - name: Install frontend dependencies
        run: cd frontend && pnpm install --frozen-lockfile

      - name: Validate frontend lint
        run: make frontend-lint

  frontend-test:
    name: Frontend test
    runs-on: ubuntu-latest
    timeout-minutes: 15

    steps:
      - name: Checkout
        uses: actions/checkout@v4

      - name: Install Node
        uses: actions/setup-node@v4
        with:
          node-version: '24'

      - name: Install pnpm from frontend package
        run: |
          corepack enable
          PNPM_VERSION="$(node -p "require('./frontend/package.json').packageManager.split('@')[1]")"
          corepack prepare "pnpm@${PNPM_VERSION}" --activate

      - name: Install frontend dependencies
        run: cd frontend && pnpm install --frozen-lockfile

      - name: Validate frontend tests
        run: make frontend-test

  frontend-build:
    name: Frontend build
    runs-on: ubuntu-latest
    timeout-minutes: 15

    steps:
      - name: Checkout
        uses: actions/checkout@v4

      - name: Install Node
        uses: actions/setup-node@v4
        with:
          node-version: '24'

      - name: Install pnpm from frontend package
        run: |
          corepack enable
          PNPM_VERSION="$(node -p "require('./frontend/package.json').packageManager.split('@')[1]")"
          corepack prepare "pnpm@${PNPM_VERSION}" --activate

      - name: Install frontend dependencies
        run: cd frontend && pnpm install --frozen-lockfile

      - name: Validate frontend build
        run: make frontend-build
```

### `.github/workflows/nightly.yml`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/.github/workflows/nightly.yml`
- Size bytes / Размер в байтах: `1410`
- Included characters / Включено символов: `1410`
- Truncated / Обрезано: `no`

```yaml
name: Nightly quality

on:
  schedule:
    - cron: '0 3 * * *'
  workflow_dispatch:

jobs:
  backend-e2e:
    name: Backend e2e
    runs-on: ubuntu-latest
    timeout-minutes: 60
    env:
      CARGO_TARGET_ROOT: /tmp/makosh-cargo-target-${{ github.job }}

    steps:
      - name: Checkout
        uses: actions/checkout@v4

      - name: Install Rust toolchain
        run: |
          rustup toolchain install 1.88 --profile minimal
          rustup default 1.88

      - name: Install Node
        uses: actions/setup-node@v4
        with:
          node-version: '24'

      - name: Check Docker Compose
        run: docker compose version

      - name: Install cargo nextest
        uses: taiki-e/install-action@v2
        with:
          tool: cargo-nextest

      - name: Run backend e2e tests
        run: make test-e2e

  mutants:
    name: Mutation testing
    runs-on: ubuntu-latest
    timeout-minutes: 180
    env:
      CARGO_TARGET_ROOT: /tmp/makosh-cargo-target-${{ github.job }}

    steps:
      - name: Checkout
        uses: actions/checkout@v4

      - name: Install Rust toolchain
        run: |
          rustup toolchain install 1.88 --profile minimal
          rustup default 1.88

      - name: Install mutation toolchain
        uses: taiki-e/install-action@v2
        with:
          tool: cargo-nextest,cargo-mutants

      - name: Run mutation testing
        run: make mutants
```

### `.github/workflows/pages.yml`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/.github/workflows/pages.yml`
- Size bytes / Размер в байтах: `817`
- Included characters / Включено символов: `817`
- Truncated / Обрезано: `no`

```yaml
name: Documentation Pages

on:
  push:
    branches:
      - main
    paths:
      - 'docs/site/**'
      - '.github/workflows/pages.yml'
  workflow_dispatch:

permissions:
  contents: read
  pages: write
  id-token: write

concurrency:
  group: github-pages
  cancel-in-progress: false

jobs:
  deploy:
    name: Deploy documentation portal
    runs-on: ubuntu-latest
    environment:
      name: github-pages
      url: ${{ steps.deployment.outputs.page_url }}

    steps:
      - name: Checkout
        uses: actions/checkout@v4

      - name: Configure Pages
        uses: actions/configure-pages@v5

      - name: Upload Pages artifact
        uses: actions/upload-pages-artifact@v3
        with:
          path: docs/site

      - name: Deploy Pages
        id: deployment
        uses: actions/deploy-pages@v4
```
