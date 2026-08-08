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

- Chunk ID / ID чанка: `163-test-scripts`
- Group / Группа: `scripts`
- Role / Роль: `test`
- Status / Статус: `pending`
- Repository / Репозиторий: `/Users/avm/projects/Personal/makosh`
- Wiki path / Путь wiki: `/Users/avm/projects/Personal/makosh/docs/wiki`
- Metadata path / Путь metadata: `/Users/avm/projects/Personal/makosh/docs/wiki/_meta`
- Plan generated at / План создан: `2026-06-28T19:48:55Z`
- Per-file source limit / Лимит источника на файл: `12000` characters

## Target pages / Целевые страницы

- `operations/scripts-tests.md`

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

### `scripts/test/analyze-nextest-junit.mjs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/scripts/test/analyze-nextest-junit.mjs`
- Size bytes / Размер в байтах: `4577`
- Included characters / Включено символов: `4577`
- Truncated / Обрезано: `no`

```javascript
#!/usr/bin/env node

import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import path from "node:path";

function parseArgs(argv) {
  const args = {};
  for (let index = 0; index < argv.length; index += 2) {
    const key = argv[index];
    const value = argv[index + 1];
    if (!key?.startsWith("--") || value === undefined) {
      continue;
    }
    args[key.slice(2)] = value;
  }
  return args;
}

function parseAttributes(attributeText) {
  const attributes = {};
  const attributePattern = /([A-Za-z_:][A-Za-z0-9_.:-]*)="([^"]*)"/g;
  for (const match of attributeText.matchAll(attributePattern)) {
    attributes[match[1]] = match[2];
  }
  return attributes;
}

function percentile(sortedValues, ratio) {
  if (sortedValues.length === 0) {
    return 0;
  }
  const index = Math.min(
    sortedValues.length - 1,
    Math.max(0, Math.ceil(sortedValues.length * ratio) - 1),
  );
  return sortedValues[index];
}

function progressBar(completed, total, width = 30) {
  if (total === 0) {
    return `[${"-".repeat(width)}]`;
  }
  const filled = Math.min(width, Math.max(0, Math.round((completed / total) * width)));
  return `[${"#".repeat(filled)}${"-".repeat(width - filled)}]`;
}

const args = parseArgs(process.argv.slice(2));
const input = args.input;
const outputBase = args.output;
const suiteName = args.suite ?? "unknown";

if (!input || !outputBase) {
  console.error("Usage: analyze-nextest-junit.mjs --input <junit.xml> --output <path-prefix> [--suite <name>]");
  process.exit(2);
}

const xml = readFileSync(input, "utf8");
const testCasePattern = /<testcase\b([^>]*)>([\s\S]*?)<\/testcase>|<testcase\b([^>]*)\/>/g;
const tests = [];

for (const match of xml.matchAll(testCasePattern)) {
  const attributeText = match[1] ?? match[3] ?? "";
  const body = match[2] ?? "";
  const attrs = parseAttributes(attributeText);
  const durationSeconds = Number.parseFloat(attrs.time ?? "0");
  tests.push({
    classname: attrs.classname ?? "unknown",
    name: attrs.name ?? "unknown",
    timeSeconds: Number.isFinite(durationSeconds) ? durationSeconds : 0,
    failed: body.includes("<failure") || body.includes("<error"),
    flaky: body.includes("<flakyFailure"),
  });
}

const durations = tests.map((test) => test.timeSeconds).sort((left, right) => left - right);
const totalSeconds = durations.reduce((sum, value) => sum + value, 0);
const topSlow = [...tests]
  .sort((left, right) => right.timeSeconds - left.timeSeconds)
  .slice(0, 10)
  .map((test) => ({
    id: `${test.classname}::${test.name}`,
    timeSeconds: Number(test.timeSeconds.toFixed(3)),
    failed: test.failed,
    flaky: test.flaky,
  }));

const report = {
  suite: suiteName,
  generatedAt: new Date().toISOString(),
  source: path.relative(process.cwd(), input),
  totalTests: tests.length,
  failedTests: tests.filter((test) => test.failed).length,
  flakyTests: tests.filter((test) => test.flaky).map((test) => `${test.classname}::${test.name}`),
  totalSeconds: Number(totalSeconds.toFixed(3)),
  averageSeconds: Number((tests.length === 0 ? 0 : totalSeconds / tests.length).toFixed(3)),
  p95Seconds: Number(percentile(durations, 0.95).toFixed(3)),
  p99Seconds: Number(percentile(durations, 0.99).toFixed(3)),
  slowest: topSlow,
};

const markdown = [
  `# ${suiteName} nextest report`,
  "",
  `- Generated at: ${report.generatedAt}`,
  `- Source JUnit: \`${report.source}\``,
  `- Total tests: ${report.totalTests}`,
  `- Failed tests: ${report.failedTests}`,
  `- Flaky tests: ${report.flakyTests.length === 0 ? "none detected" : report.flakyTests.length}`,
  `- Total time: ${report.totalSeconds}s`,
  `- Average time: ${report.averageSeconds}s`,
  `- p95: ${report.p95Seconds}s`,
  `- p99: ${report.p99Seconds}s`,
  "",
  "## Slowest tests",
  "",
  ...report.slowest.map(
    (test, index) =>
      `${index + 1}. \`${test.id}\` - ${test.timeSeconds}s${test.flaky ? " (flaky)" : ""}${test.failed ? " (failed)" : ""}`,
  ),
  "",
].join("\n");

mkdirSync(path.dirname(outputBase), { recursive: true });
writeFileSync(`${outputBase}.json`, `${JSON.stringify(report, null, 2)}\n`);
writeFileSync(`${outputBase}.md`, markdown);

const passedTests = report.totalTests - report.failedTests;
const reportPath = path.relative(process.cwd(), `${outputBase}.md`);
console.log(
  `${suiteName} ${progressBar(report.totalTests, report.totalTests)} ${report.totalTests}/${report.totalTests} completed | passed ${passedTests} | failed ${report.failedTests} | flaky ${report.flakyTests.length} | total ${report.totalSeconds}s`,
);
console.log(`Report: ${reportPath}`);
```

### `scripts/test/backend-test-targets.mjs`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/scripts/test/backend-test-targets.mjs`
- Size bytes / Размер в байтах: `2768`
- Included characters / Включено символов: `2768`
- Truncated / Обрезано: `no`

```javascript
#!/usr/bin/env node

import { readdirSync, readFileSync } from "node:fs";
import path from "node:path";

const repoRoot = process.cwd();
const testsDir = path.join(repoRoot, "backend", "tests");

const TOP_LEVEL_TARGETS = readdirSync(testsDir, { withFileTypes: true })
  .filter((entry) => entry.isFile() && entry.name.endsWith(".rs"))
  .map((entry) => entry.name.replace(/\.rs$/, ""))
  .sort();

function categoryForTarget(target) {
  if (target.includes("architecture")) {
    return "architecture";
  }

  if (target.includes("snapshot")) {
    return "snapshot";
  }

  if (
    target.endsWith("_api") ||
    target.endsWith("_stream_api") ||
    target.endsWith("_websocket_api") ||
    target.endsWith("_long_poll_api") ||
    target.includes("connectrpc") ||
    target === "hard_v1_routes" ||
    target === "omniroute"
  ) {
    return "e2e";
  }

  return "integration";
}

function countRustTests(directories) {
  const pattern = /#\[(?:tokio::)?test\]/g;
  let count = 0;
  for (const directory of directories) {
    const output = BunLikeWalk(directory);
    for (const file of output) {
      if (!file.endsWith(".rs")) {
        continue;
      }
      const content = readFileSync(file, "utf8");
      count += [...content.matchAll(pattern)].length;
    }
  }
  return count;
}

function BunLikeWalk(directory) {
  const entries = readdirSync(directory, { withFileTypes: true });
  const files = [];
  for (const entry of entries) {
    const fullPath = path.join(directory, entry.name);
    if (entry.isDirectory()) {
      files.push(...BunLikeWalk(fullPath));
    } else {
      files.push(fullPath);
    }
  }
  return files;
}

const groupedTargets = {
  architecture: [],
  snapshot: [],
  e2e: [],
  integration: [],
};

for (const target of TOP_LEVEL_TARGETS) {
  groupedTargets[categoryForTarget(target)].push(target);
}

const summary = {
  generatedAt: new Date().toISOString(),
  rustUnitTestAttributes: countRustTests([
    path.join(repoRoot, "backend", "src"),
    path.join(repoRoot, "crates", "testkit", "src"),
  ]),
  backendIntegrationTargets: TOP_LEVEL_TARGETS.length,
  categories: Object.fromEntries(
    Object.entries(groupedTargets).map(([category, targets]) => [
      category,
      {
        count: targets.length,
        targets,
      },
    ]),
  ),
};

const mode = process.argv[2] ?? "summary";
const requestedCategory = process.argv[3];

if (mode === "targets") {
  if (!requestedCategory || !groupedTargets[requestedCategory]) {
    console.error("Usage: backend-test-targets.mjs targets <architecture|snapshot|e2e|integration>");
    process.exit(2);
  }

  process.stdout.write(groupedTargets[requestedCategory].join(" "));
  process.exit(0);
}

process.stdout.write(`${JSON.stringify(summary, null, 2)}\n`);
```

### `scripts/test/collect-performance-reports.sh`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/scripts/test/collect-performance-reports.sh`
- Size bytes / Размер в байтах: `795`
- Included characters / Включено символов: `795`
- Truncated / Обрезано: `no`

```bash
#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"

cd "${REPO_ROOT}"

declare -A SUITES=(
	["target/nextest/default/junit.xml"]="default"
	["target/nextest/ci/junit.xml"]="ci"
	["target/nextest/integration/junit.xml"]="integration"
)

found_any=0

for input in "${!SUITES[@]}"; do
	if [[ ! -f "${input}" ]]; then
		continue
	fi

	found_any=1
	node scripts/test/analyze-nextest-junit.mjs \
		--input "${input}" \
		--suite "${SUITES[${input}]}" \
		--output "reports/test-performance/${SUITES[${input}]}"
done

if [[ "${found_any}" -eq 0 ]]; then
	echo "No nextest JUnit XML files found under target/nextest/" >&2
	echo "Run a nextest-based command first, for example: make test-unit" >&2
	exit 1
fi
```

### `scripts/test/run-llvm-cov.sh`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/scripts/test/run-llvm-cov.sh`
- Size bytes / Размер в байтах: `916`
- Included characters / Включено символов: `916`
- Truncated / Обрезано: `no`

```bash
#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"

source "${REPO_ROOT}/scripts/lib/rust-tooling.sh"

PROFILE="${1:-ci}"
shift || true

require_cargo_subcommand "llvm-cov" "cargo install --locked cargo-llvm-cov"
require_cargo_subcommand "nextest" "cargo install --locked cargo-nextest"

CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-${REPO_ROOT}/target/coverage-build}"
NEXTEST_SHOW_PROGRESS="${NEXTEST_SHOW_PROGRESS:-bar}"
export CARGO_TARGET_DIR
export CARGO_INCREMENTAL="${CARGO_INCREMENTAL:-0}"

cd "${REPO_ROOT}"

cargo llvm-cov clean --workspace
cargo run --manifest-path crates/testkit/Cargo.toml --bin makosh_test_session -- \
	cargo llvm-cov nextest \
		--manifest-path backend/Cargo.toml \
		--profile "${PROFILE}" \
		--show-progress "${NEXTEST_SHOW_PROGRESS}" \
		--test-threads "${MAKOSH_NEXTEST_JOBS:-4}" \
		"$@"
```

### `scripts/test/run-nextest.sh`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/scripts/test/run-nextest.sh`
- Size bytes / Размер в байтах: `806`
- Included characters / Включено символов: `806`
- Truncated / Обрезано: `no`

```bash
#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"

source "${REPO_ROOT}/scripts/lib/rust-tooling.sh"

PROFILE="${1:-default}"
shift || true

require_cargo_subcommand "nextest" "cargo install --locked cargo-nextest"

CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-${REPO_ROOT}/target/validate-test}"
NEXTEST_SHOW_PROGRESS="${NEXTEST_SHOW_PROGRESS:-bar}"
export CARGO_TARGET_DIR
export CARGO_INCREMENTAL="${CARGO_INCREMENTAL:-0}"

cd "${REPO_ROOT}"

cargo run --manifest-path crates/testkit/Cargo.toml --bin makosh_test_session -- \
	cargo nextest run \
		--manifest-path backend/Cargo.toml \
		--profile "${PROFILE}" \
		--show-progress "${NEXTEST_SHOW_PROGRESS}" \
		--test-threads "${MAKOSH_NEXTEST_JOBS:-4}" \
		"$@"
```
