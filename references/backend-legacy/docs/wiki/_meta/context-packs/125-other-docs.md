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

- Chunk ID / ID чанка: `125-other-docs`
- Group / Группа: `docs`
- Role / Роль: `other`
- Status / Статус: `pending`
- Repository / Репозиторий: `/Users/avm/projects/Personal/makosh`
- Wiki path / Путь wiki: `/Users/avm/projects/Personal/makosh/docs/wiki`
- Metadata path / Путь metadata: `/Users/avm/projects/Personal/makosh/docs/wiki/_meta`
- Plan generated at / План создан: `2026-06-28T19:48:55Z`
- Per-file source limit / Лимит источника на файл: `12000` characters

## Target pages / Целевые страницы

- `components/docs.md`

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

### `docs/site/assets/makosh-logo-mark.png`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/site/assets/makosh-logo-mark.png`
- Size bytes / Размер в байтах: `106906`
- Included characters / Включено символов: `0`
- Truncated / Обрезано: `not_applicable_binary`

_Binary file content omitted; use the path and metadata only. / Содержимое бинарного файла не включено; используй только путь и metadata._

### `docs/site/makosh-docs.css`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/site/makosh-docs.css`
- Size bytes / Размер в байтах: `7653`
- Included characters / Включено символов: `7653`
- Truncated / Обрезано: `no`

```text
:root {
	--hh-font-sans:
		Inter, 'SF Pro Display', ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont,
		'Segoe UI', sans-serif;
	--hh-color-bg: #02090b;
	--hh-color-bg-raised: #020d10;
	--hh-color-surface: #06181b;
	--hh-color-surface-deep: #041215;
	--hh-color-text: #eefefb;
	--hh-color-text-strong: #f2fffd;
	--hh-color-text-muted: #91a8a8;
	--hh-color-text-dim: #849ca0;
	--hh-color-accent: #2df0ce;
	--hh-color-accent-strong: #25d8bd;
	--hh-color-accent-contrast: #032522;
	--hh-border-accent-soft: rgba(45, 240, 206, 0.18);
	--hh-border-accent: rgba(45, 240, 206, 0.42);
	--hh-border-subtle: rgba(111, 205, 195, 0.14);
	--hh-surface-panel: rgba(8, 29, 33, 0.94);
	--hh-accent-tint: rgba(45, 240, 206, 0.08);
	--hh-radius-control: 7px;
	--hh-radius-md: 8px;
	--hh-radius-round: 50%;
	--hh-radius-sidebar: 0 18px 34px 0;
	--hh-shadow-sidebar:
		inset -1px 0 0 rgba(255, 255, 255, 0.03), 18px 0 48px rgba(0, 0, 0, 0.28);
	--hh-shadow-panel: inset 0 1px 0 rgba(255, 255, 255, 0.035);
}

* {
	box-sizing: border-box;
}

html {
	min-width: 320px;
	background: var(--hh-color-bg);
	scroll-behavior: smooth;
}

body {
	margin: 0;
	min-height: 100vh;
	background:
		linear-gradient(rgba(2, 9, 11, 0.46), rgba(2, 9, 11, 0.46)),
		radial-gradient(circle at 72% 2%, rgba(23, 122, 121, 0.14), transparent 34%),
		linear-gradient(180deg, rgba(7, 28, 32, 0.88), rgba(2, 9, 11, 0.98) 46%),
		var(--hh-color-bg);
	color: var(--hh-color-text);
	font-family: var(--hh-font-sans);
	letter-spacing: 0;
}

a {
	color: inherit;
	text-decoration: none;
}

a:focus-visible {
	outline: 1px solid rgba(45, 240, 206, 0.62);
	outline-offset: 2px;
}

.docs-shell {
	display: grid;
	grid-template-columns: 224px minmax(0, 1fr);
	gap: 16px;
	min-height: 100vh;
	padding: 0 14px 16px 0;
}

.docs-sidebar {
	position: sticky;
	top: 0;
	display: grid;
	grid-template-rows: auto 1fr;
	align-self: start;
	min-height: 100vh;
	padding: 24px 12px 16px;
	border: 1px solid rgba(37, 224, 197, 0.14);
	border-left: 0;
	border-radius: var(--hh-radius-sidebar);
	background:
		linear-gradient(180deg, rgba(4, 26, 29, 0.7), rgba(2, 13, 16, 0.56)),
		var(--hh-color-bg-raised);
	backdrop-filter: blur(12px);
	-webkit-backdrop-filter: blur(12px);
	box-shadow: var(--hh-shadow-sidebar);
}

.brand {
	display: grid;
	grid-template-columns: 32px minmax(0, 1fr);
	gap: 12px;
	align-items: center;
	padding: 0 4px 24px;
}

.brand-mark {
	width: 32px;
	height: 32px;
	object-fit: contain;
	filter: drop-shadow(0 0 16px rgba(37, 224, 197, 0.55));
}

.brand-copy {
	display: grid;
	min-width: 0;
	gap: 2px;
}

.brand-copy strong {
	overflow: hidden;
	color: var(--hh-color-text-strong);
	font-size: 15px;
	font-weight: 600;
	text-transform: uppercase;
	text-overflow: ellipsis;
	white-space: nowrap;
}

.brand-copy span {
	overflow: hidden;
	color: var(--hh-color-text-dim);
	font-size: 10px;
	font-weight: 700;
	text-transform: uppercase;
	text-overflow: ellipsis;
	white-space: nowrap;
}

.nav-group {
	display: grid;
	align-content: start;
	gap: 4px;
}

.nav-group a {
	display: grid;
	align-items: center;
	min-height: 32px;
	border: 1px solid transparent;
	border-radius: var(--hh-radius-control);
	color: #c6d7d7;
	padding: 0 8px;
	font-size: 13px;
	transition:
		border-color 180ms ease-out,
		background 180ms ease-out,
		box-shadow 180ms ease-out,
		color 180ms ease-out;
}

.nav-group a:hover,
.nav-group a:focus-visible {
	border-color: rgba(45, 240, 206, 0.18);
	background: rgba(27, 94, 85, 0.16);
	box-shadow: inset 0 0 18px rgba(45, 240, 206, 0.08);
	color: #e5fffb;
}

.nav-group a.active {
	border-color: rgba(40, 236, 205, 0.45);
	background: linear-gradient(90deg, rgba(12, 112, 93, 0.54), rgba(8, 54, 54, 0.34));
	box-shadow:
		inset 0 0 24px rgba(28, 221, 188, 0.14),
		0 0 18px rgba(24, 189, 164, 0.08);
	color: #40f3d1;
}

.docs-main {
	display: grid;
	align-content: start;
	gap: 16px;
	min-width: 0;
	padding: 16px 0 0;
}

.topbar {
	display: flex;
	flex-wrap: wrap;
	gap: 8px;
	align-items: center;
	min-height: 37px;
	border: 1px solid var(--hh-border-subtle);
	border-radius: var(--hh-radius-md);
	background:
		linear-gradient(180deg, rgba(8, 31, 35, 0.78), rgba(4, 18, 21, 0.72)),
		var(--hh-color-surface-deep);
	padding: 8px;
	box-shadow: var(--hh-shadow-panel);
}

.topbar span,
.flow-strip span {
	display: inline-flex;
	align-items: center;
	min-height: 26px;
	border: 1px solid rgba(45, 240, 206, 0.16);
	border-radius: var(--hh-radius-control);
	background: rgba(25, 154, 132, 0.14);
	color: var(--hh-color-accent);
	padding: 0 10px;
	font-size: 12px;
	font-weight: 700;
}

.intro-panel,
.panel {
	min-width: 0;
	border: 1px solid var(--hh-border-subtle);
	border-radius: var(--hh-radius-md);
	background:
		linear-gradient(180deg, rgba(8, 31, 35, 0.82), rgba(4, 18, 21, 0.76)),
		var(--hh-color-surface);
	box-shadow: var(--hh-shadow-panel);
}

.intro-panel {
	display: grid;
	gap: 12px;
	padding: 24px;
}

.panel {
	display: grid;
	gap: 12px;
	padding: 18px;
}

.eyebrow {
	margin: 0;
	color: var(--hh-color-accent);
	font-size: 11px;
	font-weight: 800;
	text-transform: uppercase;
}

h1,
h2 {
	margin: 0;
	color: var(--hh-color-text-strong);
	line-height: 1.12;
}

h1 {
	max-width: 760px;
	font-size: 34px;
	font-weight: 760;
}

h2 {
	font-size: 18px;
	font-weight: 720;
}

.lead {
	max-width: 820px;
	margin: 0;
	color: var(--hh-color-text-soft, #dcefed);
	font-size: 15px;
	line-height: 1.6;
}

.flow-strip {
	display: flex;
	flex-wrap: wrap;
	gap: 8px;
	align-items: center;
}

.flow-strip span {
	position: relative;
}

.grid {
	display: grid;
	gap: 16px;
	min-width: 0;
}

.grid.two {
	grid-template-columns: repeat(2, minmax(0, 1fr));
}

.link-list,
.compact-links,
.tile-grid {
	display: grid;
	gap: 8px;
	min-width: 0;
}

.link-list a,
.compact-links a,
.tile-grid a {
	display: grid;
	align-items: center;
	min-width: 0;
	min-height: 34px;
	border: 1px solid rgba(45, 240, 206, 0.12);
	border-radius: var(--hh-radius-control);
	background: rgba(5, 22, 25, 0.72);
	color: #dff8f4;
	padding: 0 10px;
	font-size: 13px;
	overflow-wrap: anywhere;
	transition:
		border-color 160ms ease-out,
		background 160ms ease-out,
		color 160ms ease-out;
}

.link-list a:hover,
.compact-links a:hover,
.tile-grid a:hover {
	border-color: var(--hh-border-accent);
	background: var(--hh-accent-tint);
	color: var(--hh-color-accent);
}

.tile-grid {
	grid-template-columns: repeat(5, minmax(0, 1fr));
}

.compact-links {
	grid-template-columns: repeat(2, minmax(0, 1fr));
}

.plain-list {
	display: grid;
	gap: 8px;
	margin: 0;
	padding: 0;
	list-style: none;
	color: #d6ebe8;
	font-size: 13px;
	line-height: 1.45;
}

.plain-list li {
	min-width: 0;
	border-left: 2px solid rgba(45, 240, 206, 0.32);
	padding-left: 10px;
	overflow-wrap: anywhere;
}

code {
	border: 1px solid rgba(45, 240, 206, 0.16);
	border-radius: 5px;
	background: rgba(45, 240, 206, 0.08);
	color: var(--hh-color-accent);
	padding: 1px 5px;
	font-family: 'SFMono-Regular', Consolas, 'Liberation Mono', monospace;
	font-size: 12px;
}

@media (max-width: 1100px) {
	.tile-grid {
		grid-template-columns: repeat(3, minmax(0, 1fr));
	}
}

@media (max-width: 840px) {
	.docs-shell {
		grid-template-columns: 1fr;
		padding: 0 12px 16px;
	}

	.docs-sidebar {
		position: relative;
		min-height: auto;
		border-left: 1px solid rgba(37, 224, 197, 0.14);
		border-radius: var(--hh-radius-md);
	}

	.nav-group {
		grid-template-columns: repeat(2, minmax(0, 1fr));
	}

	.grid.two {
		grid-template-columns: 1fr;
	}
}

@media (max-width: 620px) {
	.tile-grid,
	.compact-links,
	.nav-group {
		grid-template-columns: 1fr;
	}

	.intro-panel,
	.panel {
		padding: 14px;
	}

	h1 {
		font-size: 26px;
	}
}
```

### `docs/site/index.html`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/docs/site/index.html`
- Size bytes / Размер в байтах: `8283`
- Included characters / Включено символов: `8283`
- Truncated / Обрезано: `no`

```text
<!doctype html>
<html lang="en">
	<head>
		<meta charset="utf-8" />
		<meta name="viewport" content="width=device-width, initial-scale=1" />
		<title>Макошь Documentation</title>
		<meta
			name="description"
			content="Макошь documentation portal for the Personal Memory System model, domains, engines, workflows and ADRs."
		/>
		<link rel="stylesheet" href="makosh-docs.css" />
	</head>
	<body>
		<div class="docs-shell">
			<aside class="docs-sidebar" aria-label="Documentation navigation">
				<a class="brand" href="#top" aria-label="Макошь documentation home">
					<img
						class="brand-mark"
						src="assets/makosh-logo-mark.png"
						alt=""
					/>
					<span class="brand-copy">
						<strong>Макошь</strong>
						<span>Documentation</span>
					</span>
				</a>

				<nav class="nav-group" aria-label="Primary documentation sections">
					<a href="#model" class="active">Model</a>
					<a href="#entrypoints">Entrypoints</a>
					<a href="#domains">Domains</a>
					<a href="#engines">Engines</a>
					<a href="#workflows">Workflows</a>
					<a href="#implementation">Implementation</a>
					<a href="#refactoring">Refactoring</a>
				</nav>
			</aside>

			<main class="docs-main" id="top">
				<section class="topbar" aria-label="Documentation status">
					<span>Local-first</span>
					<span>Evidence-backed</span>
					<span>Event-sourced</span>
					<span>Persona-native target model</span>
				</section>

				<section class="intro-panel" id="model">
					<p class="eyebrow">Canonical model</p>
					<h1>Personal Memory System</h1>
					<p class="lead">
						Макошь stores context about communications, knowledge, memory,
						relationships, projects, documents, decisions, obligations and the
						owner's operating context.
					</p>
					<div class="flow-strip" aria-label="Communication spine">
						<span>Communication</span>
						<span>Source Evidence</span>
						<span>Knowledge</span>
						<span>Memory</span>
						<span>Context</span>
					</div>
				</section>

				<section class="grid two" id="entrypoints" aria-labelledby="entrypoints-title">
					<div class="panel">
						<p class="eyebrow">Start here</p>
						<h2 id="entrypoints-title">Canonical Entrypoints</h2>
						<div class="link-list">
							<a href="https://github.com/Mesteriis/makosh-os/blob/main/docs/product/master-spec.md">Product Master Spec</a>
							<a href="https://github.com/Mesteriis/makosh-os/blob/main/docs/foundation/vision.md">Foundation Vision</a>
							<a href="https://github.com/Mesteriis/makosh-os/blob/main/docs/foundation/glossary.md">Glossary</a>
							<a href="https://github.com/Mesteriis/makosh-os/blob/main/docs/foundation/world-model.md">World Model</a>
							<a href="https://github.com/Mesteriis/makosh-os/blob/main/docs/README.md">Documentation Index</a>
						</div>
					</div>

					<div class="panel">
						<p class="eyebrow">Current rule</p>
						<h2>What Макошь Is Not</h2>
						<ul class="plain-list">
							<li>Email client</li>
							<li>CRM or contact manager</li>
							<li>Task tracker</li>
							<li>Calendar app</li>
							<li>Note-taking app</li>
						</ul>
					</div>
				</section>

				<section class="panel" id="domains" aria-labelledby="domains-title">
					<p class="eyebrow">Durable entities</p>
					<h2 id="domains-title">Domains</h2>
					<div class="tile-grid">
						<a href="https://github.com/Mesteriis/makosh-os/blob/main/docs/domains/persons/README.md">Personas</a>
						<a href="https://github.com/Mesteriis/makosh-os/blob/main/docs/domains/communications/README.md">Communications</a>
						<a href="https://github.com/Mesteriis/makosh-os/blob/main/docs/domains/organizations/spec.md">Organizations</a>
						<a href="https://github.com/Mesteriis/makosh-os/blob/main/docs/domains/projects/README.md">Projects</a>
						<a href="https://github.com/Mesteriis/makosh-os/blob/main/docs/domains/documents/README.md">Documents</a>
						<a href="https://github.com/Mesteriis/makosh-os/blob/main/docs/domains/tasks/spec.md">Tasks</a>
						<a href="https://github.com/Mesteriis/makosh-os/blob/main/docs/domains/calendar/spec.md">Calendar and Events</a>
						<a href="https://github.com/Mesteriis/makosh-os/blob/main/docs/domains/decisions/README.md">Decisions</a>
						<a href="https://github.com/Mesteriis/makosh-os/blob/main/docs/domains/obligations/README.md">Obligations</a>
						<a href="https://github.com/Mesteriis/makosh-os/blob/main/docs/domains/graph/README.md">Knowledge Graph</a>
					</div>
				</section>

				<section class="grid two">
					<div class="panel" id="engines" aria-labelledby="engines-title">
						<p class="eyebrow">Derived mechanisms</p>
						<h2 id="engines-title">Engines</h2>
						<div class="compact-links">
							<a href="https://github.com/Mesteriis/makosh-os/blob/main/docs/engines/memory/README.md">Memory</a>
							<a href="https://github.com/Mesteriis/makosh-os/blob/main/docs/engines/timeline/README.md">Timeline</a>
							<a href="https://github.com/Mesteriis/makosh-os/blob/main/docs/engines/trust/README.md">Trust</a>
							<a href="https://github.com/Mesteriis/makosh-os/blob/main/docs/engines/search/README.md">Search</a>
							<a href="https://github.com/Mesteriis/makosh-os/blob/main/docs/engines/enrichment/README.md">Enrichment</a>
							<a href="https://github.com/Mesteriis/makosh-os/blob/main/docs/engines/obligation/README.md">Obligation</a>
							<a href="https://github.com/Mesteriis/makosh-os/blob/main/docs/engines/risk/README.md">Risk</a>
							<a href="https://github.com/Mesteriis/makosh-os/blob/main/docs/engines/consistency/README.md">Polygraph</a>
						</div>
					</div>

					<div class="panel" id="workflows" aria-labelledby="workflows-title">
						<p class="eyebrow">Evidence flow</p>
						<h2 id="workflows-title">Workflows</h2>
						<div class="compact-links">
							<a href="https://github.com/Mesteriis/makosh-os/blob/main/docs/workflows/communication-to-knowledge.md">Communication to Knowledge</a>
							<a href="https://github.com/Mesteriis/makosh-os/blob/main/docs/workflows/communication-to-obligation.md">Communication to Obligation</a>
							<a href="https://github.com/Mesteriis/makosh-os/blob/main/docs/workflows/meeting-to-decisions.md">Meeting to Decisions</a>
							<a href="https://github.com/Mesteriis/makosh-os/blob/main/docs/workflows/document-to-context.md">Document to Context</a>
							<a href="https://github.com/Mesteriis/makosh-os/blob/main/docs/workflows/contradiction-review.md">Contradiction Review</a>
							<a href="https://github.com/Mesteriis/makosh-os/blob/main/docs/workflows/dossier-generation.md">Dossier Generation</a>
						</div>
					</div>
				</section>

				<section class="grid two">
					<div class="panel" id="implementation" aria-labelledby="implementation-title">
						<p class="eyebrow">Current implementation reality</p>
						<h2 id="implementation-title">Compatibility Surface</h2>
						<ul class="plain-list">
							<li>Active identity route: <code>/api/v1/persons/{person_id}/identity</code></li>
							<li>Historical <code>contacts</code> projection was renamed to <code>persons</code>.</li>
							<li>Protected local APIs use <code>X-Макошь-Secret</code>.</li>
							<li>New credentials use host vault storage.</li>
							<li>Email channel code remains under current mail modules.</li>
						</ul>
					</div>

					<div class="panel" id="refactoring" aria-labelledby="refactoring-title">
						<p class="eyebrow">Next work</p>
						<h2 id="refactoring-title">Refactoring Plans</h2>
						<div class="link-list">
							<a href="https://github.com/Mesteriis/makosh-os/blob/main/docs/refactoring/completion-audit.md">Completion Audit</a>
							<a href="https://github.com/Mesteriis/makosh-os/blob/main/docs/refactoring/implementation-alignment-plan.md">Implementation Alignment Plan</a>
							<a href="https://github.com/Mesteriis/makosh-os/blob/main/docs/refactoring/product-alignment-plan.md">Product Alignment Plan</a>
							<a href="https://github.com/Mesteriis/makosh-os/blob/main/docs/product/development-roadmap.md">Development Roadmap</a>
							<a href="https://github.com/Mesteriis/makosh-os/blob/main/docs/adr/README.md">ADR Index</a>
						</div>
					</div>
				</section>
			</main>
		</div>
	</body>
</html>
```
