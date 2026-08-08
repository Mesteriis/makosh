# Code Wiki RU Run Report

Дата завершения проверки: `2026-06-29T01:27:20+02:00`
Репозиторий: `/Users/avm/projects/Personal/makosh`
Ветка: `main` (`feature/telemost` также указывает на текущий HEAD)
Wiki path: `docs/wiki`
Meta path: `docs/wiki/_meta`
Batch: `batch-20260628T214902`

## Итог

Полный batch DeepSeek/OpenCode для текущего рабочего дерева выполнен и применён в иерархическую Obsidian-compatible wiki на русском языке.

- indexed files: `2951`
- redacted paths: `2`
- non-redacted files: `2949`
- context packs: `163`
- DeepSeek drafts: `163`
- drafts passed: `163`
- drafts failed: `0`
- applied detail pages: `163`
- target index pages: `16`
- unique source refs in context packs: `2949`
- duplicate source refs: `0`
- missing non-redacted files in context packs: `0`
- extra source refs: `0`
- max context-pack size: `138809` bytes
- wiki validation: `passed`

Redacted paths:

- `docker/.env.example`
- `frontend/.npmrc`

## Созданная Wiki

Главная точка входа: `docs/wiki/index.md`.

Структура не является одним большим файлом: root index ведёт в разделы `systems`, `components`, `operations`, `decisions`, `flows`, `api`, `data`, `integrations`, `glossary`. Для каждого target page создан индекс, а каждый chunk сохранён отдельной detail page с:

- резюме DeepSeek;
- предложенным русским Markdown;
- покрытием источников;
- ссылками на исходные файлы;
- drift-кандидатами.

Агрегированный отчёт по расхождениям: `docs/wiki/_meta/drift-report.md`.

- drift sections requiring review: `24`
- no-drift / insufficient-context sections omitted from main findings: `139`

## OpenCode / DeepSeek Batch

Draft artifacts:

- draft dir: `docs/wiki/_meta/drafts/batch-20260628T214902`
- log dir: `docs/wiki/_meta/opencode-logs/batch-20260628T214902`
- final manifest: `docs/wiki/_meta/drafts/batch-20260628T214902.resume2-manifest.jsonl`
- batch summary: `docs/wiki/_meta/drafts/batch-20260628T214902.summary.json`

Batch result:

- chunks: `163`
- passed: `163`
- failed: `0`

Во время batch были обработаны два guard edge case:

- `078-test-backend-part-001`: guard принял безопасный placeholder `MAKOSH_API_SECRET=...` за secret-shaped assignment; resume использовал уточнённую проверку placeholder-значений.
- `123-adr-docs-part-004`: draft использовал русские названия обязательных секций; resume использовал bilingual section validation.

## Validation

- Ran: `python3 /Users/avm/.codex/skills/code-wiki-ru/scripts/validate_wiki.py --repo /Users/avm/projects/Personal/makosh --wiki-path docs/wiki --meta-path docs/wiki/_meta`
- Result: `Wiki validation passed`

- Ran: strict context-pack coverage audit over `docs/wiki/_meta/context-packs/*.md`
- Result: `2949` unique source refs, `0` duplicates, `0` missing non-redacted files, `0` extra refs.

- Ran: draft/stderr guard audit over `docs/wiki/_meta/drafts/batch-20260628T214902` and `docs/wiki/_meta/opencode-logs/batch-20260628T214902`
- Result: `163` drafts, `163` stderr logs, `0` contract errors, `0` suspicious stderr logs.

## Notes / Risks

- Wiki content is generated from bounded model drafts and validated structurally; drift findings remain candidates until a human reviews the linked source evidence.
- Raw DeepSeek drafts are preserved under `_meta/drafts/batch-20260628T214902` as audit artifacts and are not treated as applied wiki pages by the validator.
- Model-authored Obsidian links that pointed to source-document paths outside the generated wiki were normalized to code references in applied pages; generated wiki navigation links remain Obsidian links.
- No git staging or commit was performed.
