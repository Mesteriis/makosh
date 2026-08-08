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

- Chunk ID / ID чанка: `144-source-frontend-part-004`
- Group / Группа: `frontend`
- Role / Роль: `source`
- Status / Статус: `pending`
- Repository / Репозиторий: `/Users/avm/projects/Personal/makosh`
- Wiki path / Путь wiki: `/Users/avm/projects/Personal/makosh/docs/wiki`
- Metadata path / Путь metadata: `/Users/avm/projects/Personal/makosh/docs/wiki/_meta`
- Plan generated at / План создан: `2026-06-28T19:48:55Z`
- Per-file source limit / Лимит источника на файл: `12000` characters

## Target pages / Целевые страницы

- `components/frontend.md`

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

### `frontend/src/domains/communications/components/MailResourceOverviewStrip.boundary.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/components/MailResourceOverviewStrip.boundary.test.ts`
- Size bytes / Размер в байтах: `1033`
- Included characters / Включено символов: `1033`
- Truncated / Обрезано: `no`

```typescript
import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('MailResourceOverviewStrip boundary', () => {
  it('renders mailbox-level resources without direct API access', () => {
    const source = readFileSync(new URL('./MailResourceOverviewStrip.vue', import.meta.url), 'utf8')

    expect(source).toContain('subscriptions')
    expect(source).toContain('topSenders')
    expect(source).toContain('blockers')
    expect(source).toContain('Newsletters')
    expect(source).toContain('Top senders')
    expect(source).toContain('Blockers')
    expect(source).toContain("from '@tanstack/vue-virtual'")
    expect(source).toContain('useVirtualizer')
    expect(source).toContain('hasMoreSubscriptions')
    expect(source).toContain('hasMoreTopSenders')
    expect(source).toContain('loadMoreSubscriptions')
    expect(source).toContain('loadMoreTopSenders')
    expect(source).not.toContain('../api/')
    expect(source).not.toContain('fetch(')
    expect(source).not.toContain('ApiClient')
  })
})
```

### `frontend/src/domains/communications/components/MessageAiReplyPanel.boundary.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/components/MessageAiReplyPanel.boundary.test.ts`
- Size bytes / Размер в байтах: `812`
- Included characters / Включено символов: `812`
- Truncated / Обрезано: `no`

```typescript
import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('MessageAiReplyPanel boundary', () => {
  it('renders AI reply review controls without direct API access', () => {
    const source = readFileSync(new URL('./MessageAiReplyPanel.vue', import.meta.url), 'utf8')

    expect(source).toContain('AI Reply Review')
    expect(source).toContain('selectedAiReplyTone')
    expect(source).toContain('selectedAiReplyLanguage')
    expect(source).toContain('generateAiReply')
    expect(source).toContain('useGenerateAiReplyVariantsMutation')
    expect(source).toContain('generateVariants')
    expect(source).toContain('replyVariants')
    expect(source).toContain('applyAiReply')
    expect(source).not.toContain('../api/')
    expect(source).not.toContain('fetch(')
  })
})
```

### `frontend/src/domains/communications/components/MessageAttachmentsTab.boundary.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/components/MessageAttachmentsTab.boundary.test.ts`
- Size bytes / Размер в байтах: `1705`
- Included characters / Включено символов: `1705`
- Truncated / Обрезано: `no`

```typescript
import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('MessageAttachmentsTab boundary', () => {
  it('uses TanStack Query archive inspection wiring without direct component fetch', () => {
    const source = readFileSync(
      new URL('./MessageAttachmentsTab.vue', import.meta.url),
      'utf8'
    )

    expect(source).toContain('useAttachmentArchiveInspectionQuery')
    expect(source).toContain('useAttachmentPreviewQuery')
    expect(source).toContain('useTranslateAttachmentMutation')
    expect(source).toContain('attachmentTranslationTarget')
    expect(source).toContain('attachmentTranslationResult')
    expect(source).toContain('attachmentTranslationError')
    expect(source).toContain('translateSelectedAttachment')
    expect(source).toContain('source_text: preview.text')
    expect(source).toContain('attachment-translation-panel')
    expect(source).toContain('Attachment translation')
    expect(source).toContain('isInspectableArchiveAttachment')
    expect(source).toContain('isPreviewableImageAttachment')
    expect(source).toContain('isPreviewablePdfAttachment')
    expect(source).toContain('isPreviewableAttachment')
    expect(source).toContain('Inspect archive')
    expect(source).toContain('Attachment preview')
    expect(source).toContain('attachment-preview-image')
    expect(source).toContain('attachment-preview-media')
    expect(source).toContain('attachment-preview-document')
    expect(source).toContain("attachmentPreview.preview_kind === 'pdf'")
    expect(source).toContain('attachmentPreview.data_url')
    expect(source).not.toContain('../api/communications')
    expect(source).not.toMatch(/\bfetch\s*\(/)
  })
})
```

### `frontend/src/domains/communications/components/MessageBodyTab.boundary.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/components/MessageBodyTab.boundary.test.ts`
- Size bytes / Размер в байтах: `2094`
- Included characters / Включено символов: `2094`
- Truncated / Обрезано: `no`

```typescript
import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('MessageBodyTab bilingual reply boundary', () => {
  it('mounts the bilingual reply review panel without direct API access', () => {
    const source = readFileSync(
      new URL('./MessageBodyTab.vue', import.meta.url),
      'utf8'
    )

    expect(source).toContain('./BilingualReplyPanel.vue')
    expect(source).toContain('isBilingualReplyOpen')
    expect(source).toContain('<BilingualReplyPanel')
    expect(source).toContain('sendBilingualReply')
    expect(source).toContain('messageId')
    expect(source).toContain('aiSummaryContractFromMetadata')
    expect(source).toContain('summaryContract')
    expect(source).toContain('ai-summary-contract')
    expect(source).toContain('Key points')
    expect(source).toContain('Action items')
    expect(source).toContain('Risks')
    expect(source).toContain('Deadlines')
    expect(source).toContain('communicationExtractionSectionsFromInsight')
    expect(source).toContain('communicationKnowledgeSectionsFromSummaryContract')
    expect(source).toContain('extractionSections')
    expect(source).toContain('knowledgeSections')
    expect(source).toContain('extraction-review')
    expect(source).toContain('Extraction Review')
    expect(source).toContain('knowledge-review')
    expect(source).toContain('Knowledge Review')
    expect(source).toContain('generateAiReply')
    expect(source).toContain('applyAiReply')
    expect(source).toContain('reviewSecurity')
    expect(source).toContain('reviewRecipients')
    expect(source).toContain('MessageAiReplyPanel')
    expect(source).toContain('MessageTrustReviewPanel')
    expect(source).toContain('MessageLocalIntelligencePanel')
    expect(source).toContain('remoteImageUrls')
    expect(source).toContain('shouldLoadRemoteImages')
    expect(source).toContain('remoteImageProxyUrl')
    expect(source).toContain('Remote images blocked')
    expect(source).toContain('/remote-image?url=')
    expect(source).not.toContain('../api/')
    expect(source).not.toContain('fetch(')
  })
})
```

### `frontend/src/domains/communications/components/MessageLocalIntelligencePanel.boundary.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/components/MessageLocalIntelligencePanel.boundary.test.ts`
- Size bytes / Размер в байтах: `754`
- Included characters / Включено символов: `754`
- Truncated / Обрезано: `no`

```typescript
import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('MessageLocalIntelligencePanel boundary', () => {
  it('runs explain and language detection through query hooks without direct API access', () => {
    const source = readFileSync(new URL('./MessageLocalIntelligencePanel.vue', import.meta.url), 'utf8')

    expect(source).toContain('useExplainMessageMutation')
    expect(source).toContain('useDetectMessageLanguageMutation')
    expect(source).toContain('Importance')
    expect(source).toContain('Detect language')
    expect(source).toContain('Why this matters')
    expect(source).not.toContain('../api/')
    expect(source).not.toContain('fetch(')
    expect(source).not.toContain('ApiClient')
  })

})
```

### `frontend/src/domains/communications/components/MessageRelatedTab.boundary.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/components/MessageRelatedTab.boundary.test.ts`
- Size bytes / Размер в байтах: `1775`
- Included characters / Включено символов: `1775`
- Truncated / Обрезано: `no`

```typescript
import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('MessageRelatedTab export boundary', () => {
  it('offers every message export API format without direct API access', () => {
    const source = readFileSync(new URL('./MessageRelatedTab.vue', import.meta.url), 'utf8')

    expect(source).toContain('exportMessage')
    expect(source).toContain("'md'")
    expect(source).toContain("'eml'")
    expect(source).toContain("'json'")
    expect(source).toContain('Markdown')
    expect(source).toContain('EML')
    expect(source).toContain('JSON')
    expect(source).toContain('communicationMessageLabelsFromMetadata')
    expect(source).toContain('communicationMessageSnoozeUntilFromMetadata')
    expect(source).toContain('replyAll')
    expect(source).toContain('forwardMessage')
    expect(source).toContain('redirectMessage')
    expect(source).toContain('markMessageRead')
    expect(source).toContain('markMessageUnread')
    expect(source).toContain('deleteFromProvider')
    expect(source).toContain('redirectRecipientsText')
    expect(source).toContain('Reply All')
    expect(source).toContain('Forward')
    expect(source).toContain('Redirect')
    expect(source).toContain('Read / Delete')
    expect(source).toContain("emit('markMessageRead')")
    expect(source).toContain("emit('markMessageUnread')")
    expect(source).toContain("emit('deleteFromProvider')")
    expect(source).toContain('addLabel')
    expect(source).toContain('removeLabel')
    expect(source).toContain('snoozeMessage')
    expect(source).toContain('Snooze')
    expect(source).toContain('Follow up')
    expect(source).not.toContain('../api/')
    expect(source).not.toMatch(/\bfetch\s*\(/)
    expect(source).not.toContain('ApiClient')
  })
})
```

### `frontend/src/domains/communications/components/MessageTrustReviewPanel.boundary.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/components/MessageTrustReviewPanel.boundary.test.ts`
- Size bytes / Размер в байтах: `702`
- Included characters / Включено символов: `702`
- Truncated / Обрезано: `no`

```typescript
import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('MessageTrustReviewPanel boundary', () => {
  it('renders security and recipient review controls without direct API access', () => {
    const source = readFileSync(new URL('./MessageTrustReviewPanel.vue', import.meta.url), 'utf8')

    expect(source).toContain('Security Review')
    expect(source).toContain('Recipient Suggestions')
    expect(source).toContain('reviewSecurity')
    expect(source).toContain('reviewRecipients')
    expect(source).toContain('authRisk')
    expect(source).toContain('smartCc')
    expect(source).not.toContain('../api/')
    expect(source).not.toContain('fetch(')
  })
})
```

### `frontend/src/domains/communications/components/OutboxStatusStrip.boundary.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/components/OutboxStatusStrip.boundary.test.ts`
- Size bytes / Размер в байтах: `910`
- Included characters / Включено символов: `910`
- Truncated / Обрезано: `no`

```typescript
import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('OutboxStatusStrip boundary', () => {
  it('renders existing query data without owning API or cache logic', () => {
    const source = readFileSync(
      new URL('./OutboxStatusStrip.vue', import.meta.url),
      'utf8'
    )

    expect(source).toContain('visibleOutboxStatusItems')
    expect(source).toContain('outboxStatusPresentation')
    expect(source).toContain("undo: [outboxId: string]")
    expect(source).toContain("loadMore: []")
    expect(source).toContain("prefetchMore: []")
    expect(source).toContain('v-if="hasMore"')
    expect(source).toContain('@mouseenter="emit(\'prefetchMore\')"')
    expect(source).toContain('@focus="emit(\'prefetchMore\')"')
    expect(source).not.toContain('fetch(')
    expect(source).not.toContain('ApiClient')
    expect(source).not.toContain('useQuery')
  })
})
```

### `frontend/src/domains/communications/components/RichComposeEditor.boundary.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/components/RichComposeEditor.boundary.test.ts`
- Size bytes / Размер в байтах: `3494`
- Included characters / Включено символов: `3494`
- Truncated / Обрезано: `no`

```typescript
import { readFileSync } from 'node:fs'
import { dirname, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'
import { describe, expect, it } from 'vitest'

const componentPath = resolve(dirname(fileURLToPath(import.meta.url)), 'RichComposeEditor.vue')
const schemaPath = resolve(dirname(fileURLToPath(import.meta.url)), 'richComposeExtensions.ts')

describe('RichComposeEditor boundaries', () => {
  it('uses TipTap runtime instead of browser execCommand contenteditable editing', () => {
    const source = readFileSync(componentPath, 'utf8')

    expect(source).toContain("import { richComposeExtensions } from './richComposeExtensions'")
    expect(source).toContain('useEditor')
    expect(source).toContain('EditorContent')
    expect(source).toContain('richComposeExtensions')
    expect(source).not.toContain('Node.create')
    expect(source).not.toContain('Mark.create')
    expect(source).not.toContain('document.execCommand')
    expect(source).not.toContain('contenteditable="true"')
  })

  it('keeps compose formatting commands scoped to supported mail-safe controls', () => {
    const source = readFileSync(componentPath, 'utf8')

    expect(source).toContain('linkHref')
    expect(source).toContain("runCommand('paragraph')")
    expect(source).toContain("runCommand('heading2')")
    expect(source).toContain("runCommand('heading3')")
    expect(source).toContain("runCommand('alignLeft')")
    expect(source).toContain("runCommand('alignCenter')")
    expect(source).toContain("runCommand('alignRight')")
    expect(source).toContain('updateAttributes')
    expect(source).toContain("runCommand('bold')")
    expect(source).toContain("runCommand('italic')")
    expect(source).toContain("runCommand('bulletList')")
    expect(source).toContain("runCommand('orderedList')")
    expect(source).toContain("runCommand('blockquote')")
    expect(source).toContain("runCommand('link')")
    expect(source).toContain("runCommand('unlink')")
  })

  it('keeps the local TipTap schema in a focused mail-safe extension module', () => {
    const source = readFileSync(schemaPath, 'utf8')

    expect(source).toContain("from '@tiptap/vue-3'")
    expect(source).toContain('RichHeading')
    expect(source).toContain('RichLink')
    expect(source).toContain('RichOrderedList')
    expect(source).toContain('RichBlockquote')
    expect(source).toContain('normalizeMailComposeLinkHref')
    expect(source).toContain('normalizeMailComposeTextAlign')
    expect(source).toContain('getSafeTextAlignAttributes')
    expect(source).toContain("rel: 'noopener noreferrer'")
    expect(source).toContain("target: '_blank'")
    expect(source).toContain('export const richComposeExtensions')
  })

  it('intercepts pasted and dropped HTML before TipTap inserts it into the draft', () => {
    const source = readFileSync(componentPath, 'utf8')

    expect(source).toContain('sanitizeMailComposePastedHtml')
    expect(source).toContain('handlePaste')
    expect(source).toContain('handleDrop')
    expect(source).toContain('event.preventDefault()')
    expect(source).toContain('insertContent(sanitizeMailComposePastedHtml')
  })

  it('emits dropped attachment files instead of inserting them into rich HTML', () => {
    const source = readFileSync(componentPath, 'utf8')

    expect(source).toContain("'attachments-dropped': [files: File[]]")
    expect(source).toContain("emit('attachments-dropped'")
    expect(source).toContain('Array.from(event.dataTransfer?.files')
  })
})
```

### `frontend/src/domains/communications/components/SavedSearchRuleGroupEditor.boundary.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/components/SavedSearchRuleGroupEditor.boundary.test.ts`
- Size bytes / Размер в байтах: `816`
- Included characters / Включено символов: `816`
- Truncated / Обрезано: `no`

```typescript
import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('SavedSearchRuleGroupEditor boundaries', () => {
  it('renders nested group summary cues without owning query or API logic', () => {
    const source = readFileSync(new URL('./SavedSearchRuleGroupEditor.vue', import.meta.url), 'utf8')

    expect(source).toContain("from './savedSearchRuleTreePresentation'")
    expect(source).toContain('savedSearchRuleGroupDepthLabel')
    expect(source).toContain('savedSearchRuleGroupSummary')
    expect(source).toContain('saved-search-group-builder-summary')
    expect(source).toContain(':depth="nextDepth()"')
    expect(source).toContain("{{ isRoot ? 'Match' : 'Group match' }}")
    expect(source).not.toContain('../api/')
    expect(source).not.toContain('ApiClient')
  })
})
```

### `frontend/src/domains/communications/components/SavedSearchStrip.boundary.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/components/SavedSearchStrip.boundary.test.ts`
- Size bytes / Размер в байтах: `3290`
- Included characters / Включено символов: `3290`
- Truncated / Обрезано: `no`

```typescript
import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('SavedSearchStrip prefetch boundary', () => {
  it('prefetches saved-search mail list results without direct API calls from the component', () => {
    const source = readFileSync(new URL('./SavedSearchStrip.vue', import.meta.url), 'utf8')

    expect(source).toContain("from '@tanstack/vue-virtual'")
    expect(source).toContain('useVirtualizer')
    expect(source).toContain('horizontal: true')
    expect(source).toContain('virtualSmartFolders')
    expect(source).toContain('virtualSavedSearches')
    expect(source).toContain('smartFolderVirtualTotalSize')
    expect(source).toContain('savedSearchVirtualTotalSize')
    expect(source).toContain('v-for="virtualItem in virtualSmartFolders"')
    expect(source).toContain('v-for="virtualItem in virtualSavedSearches"')
    expect(source).toContain('fetchNextPage: fetchNextSmartFolderPage')
    expect(source).toContain('fetchNextPage: fetchNextSavedSearchPage')
    expect(source).toContain('savedSearchFilterChips')
    expect(source).toContain('savedSearchPresetOptions')
    expect(source).toContain('savedSearchWorkflowOptions')
    expect(source).toContain('savedSearchLocalStateOptions')
    expect(source).toContain('savedSearchChannelOptions')
    expect(source).toContain('normalizeSavedSearchBuilderState')
    expect(source).toContain('composeSavedSearchRuleTreeQuery')
    expect(source).toContain('validateSavedSearchRuleTree')
    expect(source).toContain('SavedSearchRuleGroupEditor')
    expect(source).toContain('searchRuleTree')
    expect(source).toContain('normalizeQueryIntoBuilder(formValues.query)')
    expect(source).toContain('const effectiveQueryPreview = computed(() =>')
    expect(source).toContain('const ruleValidation = computed(() => validateSavedSearchRuleTree(searchRuleTree.value))')
    expect(source).toContain('Rules Builder')
    expect(source).toContain('Effective query')
    expect(source).toContain('saved-search-effective-query')
    expect(source).toContain('saved-search-rule-error')
    expect(source).toContain(':disabled="isSaving || !ruleValidation.isValid"')
    expect(source).toContain('currentQuery')
    expect(source).toContain('currentWorkflowState')
    expect(source).toContain('currentLocalState')
    expect(source).toContain('currentChannelKind')
    expect(source).toContain('currentSearchDefaults')
    expect(source).toContain('activeFilterChips')
    expect(source).toContain('applyPreset(preset)')
    expect(source).toContain('v-for="chip in activeFilterChips"')
    expect(source).toContain('handleSmartFolderVirtualScroll')
    expect(source).toContain('handleSavedSearchVirtualScroll')
    expect(source).toContain('@scroll="handleSmartFolderVirtualScroll"')
    expect(source).toContain('@scroll="handleSavedSearchVirtualScroll"')
    expect(source).toContain('useSavedSearchCommunicationListPrefetch')
    expect(source).toContain('handleSavedSearchPrefetch')
    expect(source).toContain('@mouseenter="handleSavedSearchPrefetch(smartFolders[virtualItem.index])"')
    expect(source).toContain('@focus="handleSavedSearchPrefetch(savedSearches[virtualItem.index])"')
    expect(source).not.toContain('../api/')
    expect(source).not.toContain('ApiClient')
  })
})
```

### `frontend/src/domains/communications/components/ThreadAttachmentInsightPanel.boundary.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/components/ThreadAttachmentInsightPanel.boundary.test.ts`
- Size bytes / Размер в байтах: `1080`
- Included characters / Включено символов: `1080`
- Truncated / Обрезано: `no`

```typescript
import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('ThreadAttachmentInsightPanel boundaries', () => {
  it('uses attachment preview and archive inspection query hooks without direct API calls', () => {
    const source = readFileSync(new URL('./ThreadAttachmentInsightPanel.vue', import.meta.url), 'utf8')

    expect(source).toContain('useAttachmentArchiveInspectionQuery')
    expect(source).toContain('useAttachmentPreviewQuery')
    expect(source).toContain('useTranslateAttachmentMutation')
    expect(source).toContain('isPreviewableAttachment')
    expect(source).toContain('isInspectableArchiveAttachment')
    expect(source).toContain('isPreviewableImageAttachment')
    expect(source).toContain('Translate preview')
    expect(source).toContain('Inspect archive')
    expect(source).toContain('Attachment preview')
    expect(source).toContain('Archive inspection')
    expect(source).toContain('Thread attachment translation')
    expect(source).not.toContain('../api/')
    expect(source).not.toContain('ApiClient')
  })
})
```

### `frontend/src/domains/communications/components/ThreadConversationView.boundary.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/components/ThreadConversationView.boundary.test.ts`
- Size bytes / Размер в байтах: `5058`
- Included characters / Включено символов: `5058`
- Truncated / Обрезано: `no`

```typescript
import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('ThreadConversationView boundary', () => {
  it('renders thread messages as a conversation timeline without fetching directly', () => {
    const source = readFileSync(
      new URL('./ThreadConversationView.vue', import.meta.url),
      'utf8'
    )

    expect(source).toContain('thread: CommunicationThreadSummary')
    expect(source).toContain('messages: ThreadMessage[]')
    expect(source).toContain('isSendingReply: boolean')
    expect(source).toContain('v-for="message in messages"')
    expect(source).toContain('expandedMessageIds')
    expect(source).toContain('activeReplyMessageId')
    expect(source).toContain('inlineReplyHtml')
    expect(source).toContain('activeReplyDraftId')
    expect(source).toContain('showQuotedContent')
    expect(source).toContain('autoExpandedThreadId')
    expect(source).toContain('expansionSummary')
    expect(source).toContain('expandedMessageCount')
    expect(source).toContain('hasQuotedMessages')
    expect(source).toContain('canExpandAllMessages')
    expect(source).toContain('canCollapseAllMessages')
    expect(source).toContain('startInlineReply')
    expect(source).toContain('cancelInlineReply')
    expect(source).toContain('continueReplyInCompose')
    expect(source).toContain('saveInlineReplyDraft')
    expect(source).toContain('sendInlineReply')
    expect(source).toContain('expandAllMessages')
    expect(source).toContain('collapseAllMessages')
    expect(source).toContain('ThreadInlineReplyComposer')
    expect(source).toContain('ThreadAttachmentInsightPanel')
    expect(source).toContain('useTranslateThreadMutation')
    expect(source).toContain('defaultExpandedThreadMessageIds')
    expect(source).toContain('hasQuotedThreadMessages')
    expect(source).toContain('summarizeThreadExpansion')
    expect(source).toContain('threadTranslationTarget')
    expect(source).toContain('threadTranslationResult')
    expect(source).toContain('threadTranslationError')
    expect(source).toContain('translatedThreadCount')
    expect(source).toContain('translatedTextForMessage')
    expect(source).toContain('handleTranslateThread')
    expect(source).toContain('props.thread.thread_id')
    expect(source).toContain('expandedMessageIds.value = new Set()')
    expect(source).toContain("autoExpandedThreadId.value = ''")
    expect(source).toContain('autoExpandedThreadId.value === props.thread.thread_id')
    expect(source).toContain('expandedMessageIds.value = defaultExpandedThreadMessageIds(messages)')
    expect(source).toContain("targetLanguage: threadTranslationTarget.value")
    expect(source).toContain('formatAttachmentSize')
    expect(source).toContain('scanStatusClass')
    expect(source).toContain('previewThreadMessageBody')
    expect(source).toContain('splitThreadMessageBody')
    expect(source).toContain('thread-translation-panel')
    expect(source).toContain('v-if="threadTranslationResult"')
    expect(source).toContain('translatedTextForMessage(message.message_id)')
    expect(source).toContain('Thread translation review')
    expect(source).toContain('Expand all')
    expect(source).toContain('Collapse all')
    expect(source).toContain("{{ showQuotedContent ? 'Hide quoted' : 'Show quoted' }}")
    expect(source).toContain('message-quoted')
    expect(source).toContain('showQuotedContent && quotedBody(message)')
    expect(source).toContain('message.attachments.length > 0')
    expect(source).toContain('message-attachment')
    expect(source).toContain('Thread message attachments')
    expect(source).toContain('<ThreadAttachmentInsightPanel :attachment="attachment" />')
    expect(source).toContain('v-model:body-html="inlineReplyHtml"')
    expect(source).toContain('@save-draft="saveInlineReplyDraft(message)"')
    expect(source).toContain('@continue-in-compose="continueReplyInCompose(message)"')
    expect(source).toContain('@send="sendInlineReply(message)"')
    expect(source).toContain('toggleMessageExpanded')
    expect(source).toContain('isMessageExpanded')
    expect(source).toContain('openMessage: [messageId: string]')
    expect(source).toContain('replyToMessage: [message: ThreadMessage, bodyHtml: string, draftId: string]')
    expect(source).toContain('saveReplyDraft: [message: ThreadMessage, bodyHtml: string, draftId: string]')
    expect(source).toContain('sendReply: [message: ThreadMessage, bodyHtml: string, draftId: string]')
    expect(source).toContain("emit('replyToMessage', message, inlineReplyHtml.value, activeReplyDraftId.value)")
    expect(source).toContain("emit('saveReplyDraft', message, inlineReplyHtml.value, activeReplyDraftId.value)")
    expect(source).toContain("emit('sendReply', message, inlineReplyHtml.value, activeReplyDraftId.value)")
    expect(source).toContain('message.body_text')
    expect(source).not.toContain('reviewingReplyMessageId')
    expect(source).not.toContain('RichComposeEditor')
    expect(source).not.toContain('fetch(')
    expect(source).not.toContain('ApiClient')
  })
})
```

### `frontend/src/domains/communications/components/ThreadInlineReplyComposer.boundary.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/components/ThreadInlineReplyComposer.boundary.test.ts`
- Size bytes / Размер в байтах: `1231`
- Included characters / Включено символов: `1231`
- Truncated / Обрезано: `no`

```typescript
import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('ThreadInlineReplyComposer boundary', () => {
  it('owns inline reply editing and send review without API or cache logic', () => {
    const source = readFileSync(
      new URL('./ThreadInlineReplyComposer.vue', import.meta.url),
      'utf8'
    )

    expect(source).toContain('message: ThreadMessage')
    expect(source).toContain('bodyHtml: string')
    expect(source).toContain('isSendingReply: boolean')
    expect(source).toContain('RichComposeEditor')
    expect(source).toContain('reviewingReply')
    expect(source).toContain('update:bodyHtml')
    expect(source).toContain('saveDraft: []')
    expect(source).toContain('continueInCompose: []')
    expect(source).toContain('send: []')
    expect(source).toContain('Review reply before sending')
    expect(source).toContain('Immediate provider send')
    expect(source).toContain("{{ isSendingReply ? 'Sending...' : 'Send' }}")
    expect(source).toContain('replyReviewRecipient')
    expect(source).toContain('replyReviewSubject')
    expect(source).not.toContain('fetch(')
    expect(source).not.toContain('ApiClient')
    expect(source).not.toContain('useQuery')
  })
})
```

### `frontend/src/domains/communications/components/attachmentSearchTable.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/components/attachmentSearchTable.test.ts`
- Size bytes / Размер в байтах: `1516`
- Included characters / Включено символов: `1516`
- Truncated / Обрезано: `no`

```typescript
import { describe, expect, it } from 'vitest'
import type { AttachmentSearchResult } from '../types/attachments'
import {
  attachmentSearchTableColumns,
  attachmentSearchTableRowId
} from './attachmentSearchTable'

function result(overrides: Partial<AttachmentSearchResult> = {}): AttachmentSearchResult {
  return {
    attachment_id: 'attachment-1',
    message_id: 'msg-1',
    raw_record_id: 'raw-1',
    account_id: 'account-1',
    message_subject: 'Invoice',
    sender: 'billing@example.com',
    occurred_at: '2026-06-15T00:00:00Z',
    blob_id: 'blob-1',
    provider_attachment_id: 'provider-1',
    filename: 'invoice.pdf',
    content_type: 'application/pdf',
    size_bytes: 2048,
    sha256: 'hash',
    disposition: 'attachment',
    scan_status: 'not_scanned',
    scan_engine: null,
    scan_checked_at: null,
    scan_summary: null,
    storage_kind: 'local',
    storage_path: 'mail/blob',
    created_at: '2026-06-15T00:00:00Z',
    updated_at: '2026-06-15T00:00:00Z',
    ...overrides
  }
}

describe('attachment search table helpers', () => {
  it('defines stable TanStack Table columns for attachment search results', () => {
    expect(attachmentSearchTableColumns.map((column) => column.id)).toEqual([
      'filename',
      'message_subject',
      'sender',
      'size',
      'scan_status'
    ])
  })

  it('uses attachment ids as stable search table row ids', () => {
    expect(attachmentSearchTableRowId(result({ attachment_id: 'attachment-42' }))).toBe('attachment-42')
  })
})
```

### `frontend/src/domains/communications/components/attachmentSearchTable.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/components/attachmentSearchTable.ts`
- Size bytes / Размер в байтах: `788`
- Included characters / Включено символов: `788`
- Truncated / Обрезано: `no`

```typescript
import type { ColumnDef } from '@tanstack/vue-table'
import type { AttachmentSearchResult } from '../types/attachments'

export const attachmentSearchTableColumns: ColumnDef<AttachmentSearchResult>[] = [
  {
    id: 'filename',
    header: 'File',
    accessorFn: (attachment) => attachment.filename || 'Unnamed'
  },
  {
    id: 'message_subject',
    header: 'Message',
    accessorKey: 'message_subject'
  },
  {
    id: 'sender',
    header: 'Sender',
    accessorKey: 'sender'
  },
  {
    id: 'size',
    header: 'Size',
    accessorFn: (attachment) => attachment.size_bytes
  },
  {
    id: 'scan_status',
    header: 'Scan',
    accessorKey: 'scan_status'
  }
]

export function attachmentSearchTableRowId(result: AttachmentSearchResult): string {
  return result.attachment_id
}
```

### `frontend/src/domains/communications/components/attachmentTable.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/components/attachmentTable.test.ts`
- Size bytes / Размер в байтах: `4849`
- Included characters / Включено символов: `4849`
- Truncated / Обрезано: `no`

```typescript
import { describe, expect, it } from 'vitest'
import type { CommunicationAttachment } from '../types/communications'
import {
  attachmentTableColumns,
  attachmentTableRowId,
  formatAttachmentSize,
  isInspectableArchiveAttachment,
  isPreviewableAttachment,
  isPreviewableImageAttachment,
  isPreviewablePdfAttachment,
  isPreviewableTextAttachment,
  scanStatusClass
} from './attachmentTable'

function attachment(overrides: Partial<CommunicationAttachment> = {}): CommunicationAttachment {
  return {
    attachment_id: 'attachment-1',
    message_id: 'msg-1',
    raw_record_id: 'raw-1',
    blob_id: 'blob-1',
    provider_attachment_id: 'provider-attachment-1',
    filename: 'invoice.pdf',
    content_type: 'application/pdf',
    size_bytes: 2048,
    sha256: 'hash',
    disposition: 'attachment',
    scan_status: 'not_scanned',
    scan_engine: null,
    scan_checked_at: null,
    scan_summary: null,
    scan_metadata: {},
    storage_kind: 'local',
    storage_path: 'mail/blob',
    created_at: '2026-06-14T10:00:00Z',
    updated_at: '2026-06-14T10:00:00Z',
    ...overrides
  }
}

describe('attachment table helpers', () => {
  it('defines stable TanStack Table columns for attachment metadata', () => {
    expect(attachmentTableColumns.map((column) => column.id)).toEqual([
      'filename',
      'content_type',
      'size',
      'scan_status'
    ])
  })

  it('uses attachment ids as stable table row ids', () => {
    expect(attachmentTableRowId(attachment({ attachment_id: 'attachment-42' }))).toBe('attachment-42')
  })

  it('formats sizes and scan status classes consistently', () => {
    expect(formatAttachmentSize(512)).toBe('512 B')
    expect(formatAttachmentSize(2048)).toBe('2.0 KB')
    expect(formatAttachmentSize(3 * 1024 * 1024)).toBe('3.0 MB')
    expect(scanStatusClass('clean')).toBe('att-scan--clean')
    expect(scanStatusClass('suspicious')).toBe('att-scan--suspicious')
    expect(scanStatusClass('malicious')).toBe('att-scan--danger')
    expect(scanStatusClass('not_scanned')).toBe('att-scan--unknown')
  })

  it('recognizes ZIP attachments as inspectable archives', () => {
    expect(isInspectableArchiveAttachment(attachment({
      filename: 'evidence.zip',
      content_type: 'application/octet-stream'
    }))).toBe(true)
    expect(isInspectableArchiveAttachment(attachment({
      filename: 'evidence.bin',
      content_type: 'application/zip'
    }))).toBe(true)
    expect(isInspectableArchiveAttachment(attachment({
      filename: 'invoice.pdf',
      content_type: 'application/pdf'
    }))).toBe(false)
  })

  it('recognizes safe text attachments as previewable', () => {
    expect(isPreviewableTextAttachment(attachment({
      filename: 'notes.txt',
      content_type: 'text/plain',
      scan_status: 'not_scanned'
    }))).toBe(true)
    expect(isPreviewableTextAttachment(attachment({
      filename: 'payload.json',
      content_type: 'application/json',
      scan_status: 'clean'
    }))).toBe(true)
    expect(isPreviewableTextAttachment(attachment({
      filename: 'danger.txt',
      content_type: 'text/plain',
      scan_status: 'malicious'
    }))).toBe(false)
    expect(isPreviewableTextAttachment(attachment({
      filename: 'invoice.pdf',
      content_type: 'application/pdf',
      scan_status: 'clean'
    }))).toBe(false)
  })

  it('recognizes safe raster image attachments as previewable', () => {
    expect(isPreviewableImageAttachment(attachment({
      filename: 'photo.png',
      content_type: 'image/png',
      scan_status: 'not_scanned'
    }))).toBe(true)
    expect(isPreviewableImageAttachment(attachment({
      filename: 'avatar.webp',
      content_type: 'application/octet-stream',
      scan_status: 'clean'
    }))).toBe(true)
    expect(isPreviewableImageAttachment(attachment({
      filename: 'unsafe.svg',
      content_type: 'image/svg+xml',
      scan_status: 'clean'
    }))).toBe(false)
    expect(isPreviewableImageAttachment(attachment({
      filename: 'danger.png',
      content_type: 'image/png',
      scan_status: 'malicious'
    }))).toBe(false)
  })

  it('recognizes safe pdf attachments as previewable', () => {
    expect(isPreviewablePdfAttachment(attachment({
      filename: 'invoice.pdf',
      content_type: 'application/pdf',
      scan_status: 'not_scanned'
    }))).toBe(true)
    expect(isPreviewablePdfAttachment(attachment({
      filename: 'invoice.pdf',
      content_type: 'application/octet-stream',
      scan_status: 'clean'
    }))).toBe(true)
    expect(isPreviewablePdfAttachment(attachment({
      filename: 'invoice.pdf',
      content_type: 'application/pdf',
      scan_status: 'malicious'
    }))).toBe(false)
    expect(isPreviewableAttachment(attachment({
      filename: 'invoice.pdf',
      content_type: 'application/pdf',
      scan_status: 'clean'
    }))).toBe(true)
  })
})
```

### `frontend/src/domains/communications/components/attachmentTable.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/components/attachmentTable.ts`
- Size bytes / Размер в байтах: `5080`
- Included characters / Включено символов: `5080`
- Truncated / Обрезано: `no`

```typescript
import type { ColumnDef } from '@tanstack/vue-table'
import type { CommunicationAttachment } from '../types/communications'

export const attachmentTableColumns: ColumnDef<CommunicationAttachment>[] = [
  {
    id: 'filename',
    header: 'File',
    accessorFn: (attachment) => attachment.filename || 'Unnamed'
  },
  {
    id: 'content_type',
    header: 'Type',
    accessorKey: 'content_type'
  },
  {
    id: 'size',
    header: 'Size',
    accessorFn: (attachment) => attachment.size_bytes
  },
  {
    id: 'scan_status',
    header: 'Scan',
    accessorKey: 'scan_status'
  }
]

export function attachmentTableRowId(attachment: CommunicationAttachment): string {
  return attachment.attachment_id
}

export function formatAttachmentSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
}

export function scanStatusClass(status: string): string {
  switch (status) {
    case 'clean': return 'att-scan--clean'
    case 'suspicious': return 'att-scan--suspicious'
    case 'malicious': return 'att-scan--danger'
    case 'failed': return 'att-scan--danger'
    default: return 'att-scan--unknown'
  }
}

export function isInspectableArchiveAttachment(attachment: CommunicationAttachment): boolean {
  const contentType = attachment.content_type.trim().toLowerCase()
  if (contentType === 'application/zip' || contentType === 'application/x-zip-compressed') {
    return true
  }
  return attachment.filename?.trim().toLowerCase().endsWith('.zip') ?? false
}

export function isPreviewableTextAttachment(attachment: CommunicationAttachment): boolean {
  if (!isPreviewAllowedByScanStatus(attachment)) {
    return false
  }
  const contentType = attachment.content_type.trim().toLowerCase()
  if (contentType.startsWith('text/')) {
    return true
  }
  if (['application/json', 'application/xml', 'application/yaml', 'application/x-yaml'].includes(contentType)) {
    return true
  }
  const filename = attachment.filename?.trim().toLowerCase()
  return Boolean(
    filename
    && (
      filename.endsWith('.txt')
      || filename.endsWith('.md')
      || filename.endsWith('.csv')
      || filename.endsWith('.json')
      || filename.endsWith('.xml')
      || filename.endsWith('.yaml')
      || filename.endsWith('.yml')
    )
  )
}

export function isPreviewableImageAttachment(attachment: CommunicationAttachment): boolean {
  if (!isPreviewAllowedByScanStatus(attachment)) {
    return false
  }
  const contentType = attachment.content_type.trim().toLowerCase()
  if (['image/png', 'image/jpeg', 'image/gif', 'image/webp'].includes(contentType)) {
    return true
  }
  const filename = attachment.filename?.trim().toLowerCase()
  return Boolean(
    filename
    && (
      filename.endsWith('.png')
      || filename.endsWith('.jpg')
      || filename.endsWith('.jpeg')
      || filename.endsWith('.gif')
      || filename.endsWith('.webp')
    )
  )
}

export function isPreviewableAudioAttachment(attachment: CommunicationAttachment): boolean {
  if (!isPreviewAllowedByScanStatus(attachment)) {
    return false
  }
  const contentType = attachment.content_type.trim().toLowerCase()
  if (contentType.startsWith('audio/')) {
    return true
  }
  const filename = attachment.filename?.trim().toLowerCase()
  return Boolean(
    filename
    && (
      filename.endsWith('.mp3')
      || filename.endsWith('.m4a')
      || filename.endsWith('.aac')
      || filename.endsWith('.ogg')
      || filename.endsWith('.opus')
      || filename.endsWith('.wav')
      || filename.endsWith('.webm')
    )
  )
}

export function isPreviewableVideoAttachment(attachment: CommunicationAttachment): boolean {
  if (!isPreviewAllowedByScanStatus(attachment)) {
    return false
  }
  const contentType = attachment.content_type.trim().toLowerCase()
  if (contentType.startsWith('video/')) {
    return true
  }
  const filename = attachment.filename?.trim().toLowerCase()
  return Boolean(
    filename
    && (
      filename.endsWith('.mp4')
      || filename.endsWith('.webm')
      || filename.endsWith('.mov')
    )
  )
}

export function isPreviewablePdfAttachment(attachment: CommunicationAttachment): boolean {
  if (!isPreviewAllowedByScanStatus(attachment)) {
    return false
  }
  const contentType = attachment.content_type.trim().toLowerCase()
  if (contentType === 'application/pdf') {
    return true
  }
  const filename = attachment.filename?.trim().toLowerCase()
  return Boolean(filename && filename.endsWith('.pdf'))
}

export function isPreviewableAttachment(attachment: CommunicationAttachment): boolean {
  return (
    isPreviewableTextAttachment(attachment) ||
    isPreviewableImageAttachment(attachment) ||
    isPreviewableAudioAttachment(attachment) ||
    isPreviewableVideoAttachment(attachment) ||
    isPreviewablePdfAttachment(attachment)
  )
}

function isPreviewAllowedByScanStatus(attachment: CommunicationAttachment): boolean {
  return ['not_scanned', 'clean'].includes(attachment.scan_status)
}
```

### `frontend/src/domains/communications/components/mailDragDrop.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/components/mailDragDrop.test.ts`
- Size bytes / Размер в байтах: `1776`
- Included characters / Включено символов: `1776`
- Truncated / Обрезано: `no`

```typescript
import { describe, expect, it } from 'vitest'
import {
  MAIL_MESSAGE_DRAG_TYPE,
  createCommunicationMessageDragPayload,
  hasCommunicationMessageDragType,
  parseCommunicationMessageDragPayload
} from './mailDragDrop'

describe('mail drag/drop helpers', () => {
  it('serializes and parses selected mail message drag payloads', () => {
    const payload = createCommunicationMessageDragPayload(' msg-1 ', [' msg-2 ', 'msg-1', ''])

    expect(parseCommunicationMessageDragPayload(payload)).toEqual({
      kind: 'mail-message-selection',
      message_id: 'msg-1',
      message_ids: ['msg-1', 'msg-2']
    })
  })

  it('keeps compatibility with legacy single-message payloads', () => {
    const payload = JSON.stringify({ kind: 'mail-message-selection', message_id: 'msg-1' })

    expect(parseCommunicationMessageDragPayload(payload)).toEqual({
      kind: 'mail-message-selection',
      message_id: 'msg-1',
      message_ids: ['msg-1']
    })
  })

  it('rejects malformed drag payloads', () => {
    expect(parseCommunicationMessageDragPayload('')).toBeNull()
    expect(parseCommunicationMessageDragPayload('not-json')).toBeNull()
    expect(parseCommunicationMessageDragPayload(JSON.stringify({ kind: 'other', message_id: 'msg-1' }))).toBeNull()
    expect(parseCommunicationMessageDragPayload(JSON.stringify({ kind: 'mail-message-selection', message_id: '' }))).toBeNull()
    expect(parseCommunicationMessageDragPayload(JSON.stringify({ kind: 'mail-message-selection', message_id: 'msg-1', message_ids: [''] }))).toBeNull()
  })

  it('detects the custom Макошь mail drag type', () => {
    expect(hasCommunicationMessageDragType([MAIL_MESSAGE_DRAG_TYPE, 'text/plain'])).toBe(true)
    expect(hasCommunicationMessageDragType(['text/plain'])).toBe(false)
  })
})
```

### `frontend/src/domains/communications/components/mailDragDrop.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/components/mailDragDrop.ts`
- Size bytes / Размер в байтах: `1882`
- Included characters / Включено символов: `1882`
- Truncated / Обрезано: `no`

```typescript
export const MAIL_MESSAGE_DRAG_TYPE = 'application/x-makosh-mail-message-selection'

export type CommunicationMessageDragPayload = {
  kind: 'mail-message-selection'
  message_id: string
  message_ids: string[]
}

export function createCommunicationMessageDragPayload(messageId: string, messageIds: string[] = []): string {
  const normalizedMessageId = messageId.trim()
  const normalizedMessageIds = uniqueNonBlankIds([normalizedMessageId, ...messageIds])
  return JSON.stringify({
    kind: 'mail-message-selection',
    message_id: normalizedMessageId,
    message_ids: normalizedMessageIds
  } satisfies CommunicationMessageDragPayload)
}

export function parseCommunicationMessageDragPayload(value: string): CommunicationMessageDragPayload | null {
  if (!value.trim()) return null

  try {
    const parsed = JSON.parse(value) as Partial<CommunicationMessageDragPayload>
    if (parsed.kind !== 'mail-message-selection') return null
    if (typeof parsed.message_id !== 'string' || !parsed.message_id.trim()) return null
    if (parsed.message_ids !== undefined && !validMessageIdList(parsed.message_ids)) return null
    const messageIds = uniqueNonBlankIds([
      parsed.message_id,
      ...(parsed.message_ids ?? [])
    ])
    return {
      kind: 'mail-message-selection',
      message_id: parsed.message_id.trim(),
      message_ids: messageIds
    }
  } catch {
    return null
  }
}

export function hasCommunicationMessageDragType(types: readonly string[] | DOMStringList): boolean {
  return Array.from(types).includes(MAIL_MESSAGE_DRAG_TYPE)
}

function validMessageIdList(value: unknown): value is string[] {
  return Array.isArray(value) && value.every((item) => typeof item === 'string' && item.trim().length > 0)
}

function uniqueNonBlankIds(values: string[]): string[] {
  return Array.from(new Set(values.map((value) => value.trim()).filter(Boolean)))
}
```

### `frontend/src/domains/communications/components/mailFolderOrdering.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/components/mailFolderOrdering.test.ts`
- Size bytes / Размер в байтах: `2768`
- Included characters / Включено символов: `2768`
- Truncated / Обрезано: `no`

```typescript
import { describe, expect, it } from 'vitest'
import type { CommunicationFolder } from '../types/folders'
import {
  createCommunicationFolderReorderPayload,
  buildCommunicationFolderReorderUpdates,
  mailFolderReorderStatus,
  parseCommunicationFolderReorderPayload
} from './mailFolderOrdering'

function folder(folderId: string, sortOrder: number): CommunicationFolder {
  return {
    folder_id: folderId,
    account_id: null,
    name: folderId,
    description: null,
    color: null,
    sort_order: sortOrder,
    message_count: 0,
    created_at: '2026-06-15T00:00:00Z',
    updated_at: '2026-06-15T00:00:00Z'
  }
}

describe('mail folder ordering', () => {
  it('moves a folder before a target with a single midpoint sort update when there is room', () => {
    const updates = buildCommunicationFolderReorderUpdates([
      folder('alpha', 1000),
      folder('bravo', 2000),
      folder('charlie', 3000)
    ], 'alpha', 'charlie')

    expect(updates).toEqual([{ folderId: 'alpha', sortOrder: 2500 }])
  })

  it('normalizes affected sort orders when no integer gap exists', () => {
    const updates = buildCommunicationFolderReorderUpdates([
      folder('alpha', 0),
      folder('bravo', 1),
      folder('charlie', 2)
    ], 'charlie', 'bravo')

    expect(updates).toEqual([
      { folderId: 'alpha', sortOrder: 1000 },
      { folderId: 'charlie', sortOrder: 2000 },
      { folderId: 'bravo', sortOrder: 3000 }
    ])
  })

  it('does not emit updates for missing folders or no-op moves', () => {
    const folders = [folder('alpha', 1000), folder('bravo', 2000)]

    expect(buildCommunicationFolderReorderUpdates(folders, 'alpha', 'alpha')).toEqual([])
    expect(buildCommunicationFolderReorderUpdates(folders, 'missing', 'alpha')).toEqual([])
    expect(buildCommunicationFolderReorderUpdates(folders, 'alpha', 'missing')).toEqual([])
  })

  it('round-trips typed drag payloads and rejects malformed payloads', () => {
    const payload = createCommunicationFolderReorderPayload(' folder-a ')

    expect(parseCommunicationFolderReorderPayload(payload)).toEqual({
      kind: 'mail-folder-reorder',
      folder_id: 'folder-a'
    })
    expect(parseCommunicationFolderReorderPayload('')).toBeNull()
    expect(parseCommunicationFolderReorderPayload('{"kind":"other","folder_id":"folder-a"}')).toBeNull()
    expect(parseCommunicationFolderReorderPayload('{"kind":"mail-folder-reorder","folder_id":" "}')).toBeNull()
  })

  it('builds reorder feedback from the payload source folder id', () => {
    expect(mailFolderReorderStatus([
      folder('alpha', 1000),
      { ...folder('charlie', 1500), name: 'Charlie' },
      { ...folder('bravo', 2000), name: 'Bravo' }
    ], 'charlie', 'bravo')).toBe('Moved Charlie before Bravo')
  })
})
```

### `frontend/src/domains/communications/components/mailFolderOrdering.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/components/mailFolderOrdering.ts`
- Size bytes / Размер в байтах: `3932`
- Included characters / Включено символов: `3932`
- Truncated / Обрезано: `no`

```typescript
import type { CommunicationFolder } from '../types/folders'

export const MAIL_FOLDER_REORDER_DRAG_TYPE = 'application/x-makosh-mail-folder-reorder'
const SORT_ORDER_STEP = 1000

export type CommunicationFolderReorderPayload = {
  kind: 'mail-folder-reorder'
  folder_id: string
}

export type CommunicationFolderOrderUpdate = {
  folderId: string
  sortOrder: number
}

export function createCommunicationFolderReorderPayload(folderId: string): string {
  return JSON.stringify({
    kind: 'mail-folder-reorder',
    folder_id: folderId.trim()
  } satisfies CommunicationFolderReorderPayload)
}

export function parseCommunicationFolderReorderPayload(value: string): CommunicationFolderReorderPayload | null {
  if (!value.trim()) return null

  try {
    const parsed = JSON.parse(value) as Partial<CommunicationFolderReorderPayload>
    if (parsed.kind !== 'mail-folder-reorder') return null
    if (typeof parsed.folder_id !== 'string' || !parsed.folder_id.trim()) return null
    return {
      kind: 'mail-folder-reorder',
      folder_id: parsed.folder_id.trim()
    }
  } catch {
    return null
  }
}

export function hasCommunicationFolderReorderDragType(types: readonly string[] | DOMStringList): boolean {
  return Array.from(types).includes(MAIL_FOLDER_REORDER_DRAG_TYPE)
}

export function buildCommunicationFolderReorderUpdates(
  folders: Pick<CommunicationFolder, 'folder_id' | 'sort_order'>[],
  sourceFolderId: string,
  targetFolderId: string
): CommunicationFolderOrderUpdate[] {
  const sourceId = sourceFolderId.trim()
  const targetId = targetFolderId.trim()
  if (!sourceId || !targetId || sourceId === targetId) return []

  const sourceIndex = folders.findIndex((folder) => folder.folder_id === sourceId)
  const targetIndex = folders.findIndex((folder) => folder.folder_id === targetId)
  if (sourceIndex < 0 || targetIndex < 0) return []

  const reordered = folders.slice()
  const [source] = reordered.splice(sourceIndex, 1)
  const adjustedTargetIndex = reordered.findIndex((folder) => folder.folder_id === targetId)
  if (!source || adjustedTargetIndex < 0) return []
  reordered.splice(adjustedTargetIndex, 0, source)

  if (sameFolderOrder(folders, reordered)) return []

  const previous = reordered[adjustedTargetIndex - 1] ?? null
  const next = reordered[adjustedTargetIndex + 1] ?? null
  const singleSortOrder = midpointSortOrder(previous?.sort_order ?? null, next?.sort_order ?? null)
  if (singleSortOrder !== null && singleSortOrder !== source.sort_order) {
    return [{ folderId: source.folder_id, sortOrder: singleSortOrder }]
  }

  return reordered.flatMap((folder, index) => {
    const sortOrder = (index + 1) * SORT_ORDER_STEP
    return sortOrder === folder.sort_order ? [] : [{ folderId: folder.folder_id, sortOrder }]
  })
}

export function mailFolderReorderStatus(
  folders: Pick<CommunicationFolder, 'folder_id' | 'name'>[],
  sourceFolderId: string,
  targetFolderId: string
): string {
  const sourceName = folders.find((folder) => folder.folder_id === sourceFolderId.trim())?.name ?? 'folder'
  const targetName = folders.find((folder) => folder.folder_id === targetFolderId.trim())?.name ?? 'folder'
  return `Moved ${sourceName} before ${targetName}`
}

function sameFolderOrder(
  left: Pick<CommunicationFolder, 'folder_id'>[],
  right: Pick<CommunicationFolder, 'folder_id'>[]
): boolean {
  return left.length === right.length && left.every((folder, index) => folder.folder_id === right[index]?.folder_id)
}

function midpointSortOrder(previous: number | null, next: number | null): number | null {
  if (previous === null && next === null) return SORT_ORDER_STEP
  if (previous === null && next !== null) return next > 0 ? next - SORT_ORDER_STEP : null
  if (previous !== null && next === null) return previous + SORT_ORDER_STEP
  if (previous === null || next === null || next - previous <= 1) return null
  return previous + Math.floor((next - previous) / 2)
}
```

### `frontend/src/domains/communications/components/mailFolderPresentation.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/components/mailFolderPresentation.test.ts`
- Size bytes / Размер в байтах: `3131`
- Included characters / Включено символов: `3131`
- Truncated / Обрезано: `no`

```typescript
import { describe, expect, it } from 'vitest'
import {
  createChildFolderDraft,
  deriveCommunicationFolderDisplayRow,
  mailFolderHierarchyDeleteImpact,
  orderCommunicationFolderDisplayRows
} from './mailFolderPresentation'
import type { CommunicationFolder } from '../types/folders'

function folder(overrides: Partial<CommunicationFolder>): CommunicationFolder {
  return {
    folder_id: 'folder-1',
    account_id: 'account-1',
    name: 'Inbox',
    description: null,
    color: '#3b82f6',
    sort_order: 0,
    message_count: 4,
    created_at: '2026-06-15T00:00:00Z',
    updated_at: '2026-06-15T00:00:00Z',
    ...overrides
  }
}

describe('mail folder presentation helpers', () => {
  it('parses root folder names as a single depth entry', () => {
    const row = deriveCommunicationFolderDisplayRow(folder({ name: 'Inbox' }))

    expect(row.depth).toBe(0)
    expect(row.leafName).toBe('Inbox')
    expect(row.pathPrefix).toBe('')
  })

  it('derives path depth, leaf and prefix from slash-delimited names', () => {
    const row = deriveCommunicationFolderDisplayRow(folder({
      folder_id: 'folder-2',
      name: 'Projects / Client A / Q1'
    }))

    expect(row.depth).toBe(2)
    expect(row.leafName).toBe('Q1')
    expect(row.pathPrefix).toBe('Projects / Client A')
  })

  it('normalizes blank segments and trims whitespace in folder paths', () => {
    const row = deriveCommunicationFolderDisplayRow(folder({
      folder_id: 'folder-3',
      name: '  Archives //  2026 // '
    }))

    expect(row.depth).toBe(1)
    expect(row.leafName).toBe('2026')
    expect(row.pathPrefix).toBe('Archives')
  })

  it('orders folders by sort order and hierarchy so parents stay ahead of children', () => {
    const rows = orderCommunicationFolderDisplayRows([
      folder({ folder_id: 'folder-4', name: 'Projects / Client A / Q1', sort_order: 100 }),
      folder({ folder_id: 'folder-2', name: 'Projects', sort_order: 100 }),
      folder({ folder_id: 'folder-3', name: 'Projects / Client A', sort_order: 100 }),
      folder({ folder_id: 'folder-1', name: 'Archive', sort_order: 50 })
    ])

    expect(rows.map((row) => row.folder.folder_id)).toEqual([
      'folder-1',
      'folder-2',
      'folder-3',
      'folder-4'
    ])
  })

  it('builds a child-folder draft from the selected parent path', () => {
    expect(createChildFolderDraft(folder({
      folder_id: 'folder-5',
      name: 'Projects / Client A',
      sort_order: 320
    }))).toEqual({
      parentPath: 'Projects / Client A',
      sortOrder: 320
    })
  })

  it('reports descendant impact for hierarchy-aware delete warnings', () => {
    expect(mailFolderHierarchyDeleteImpact([
      folder({ folder_id: 'root', name: 'Projects' }),
      folder({ folder_id: 'child-1', name: 'Projects / Client A' }),
      folder({ folder_id: 'child-2', name: 'Projects / Client B' }),
      folder({ folder_id: 'grandchild', name: 'Projects / Client A / Q1' }),
      folder({ folder_id: 'other', name: 'Archive' })
    ], 'root')).toEqual({
      descendantCount: 3,
      descendantLeafNames: ['Client A', 'Q1', 'Client B']
    })
  })
})
```

### `frontend/src/domains/communications/components/mailFolderPresentation.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/components/mailFolderPresentation.ts`
- Size bytes / Размер в байтах: `3488`
- Included characters / Включено символов: `3488`
- Truncated / Обрезано: `no`

```typescript
import type { CommunicationFolder } from '../types/folders'

export type CommunicationFolderDisplayRow = {
  folder: CommunicationFolder
  depth: number
  leafName: string
  pathPrefix: string
  pathParts: string[]
}

export type CommunicationFolderHierarchyDeleteImpact = {
  descendantCount: number
  descendantLeafNames: string[]
}

export function mailFolderColorClass(color: string | null): string {
  switch (color?.toLowerCase()) {
    case '#10b981':
      return 'mail-folder-color--green'
    case '#f59e0b':
      return 'mail-folder-color--amber'
    case '#ef4444':
      return 'mail-folder-color--red'
    case '#8b5cf6':
      return 'mail-folder-color--violet'
    default:
      return 'mail-folder-color--blue'
  }
}

export function deriveCommunicationFolderDisplayRow(folder: CommunicationFolder): CommunicationFolderDisplayRow {
  const parts = folder.name
    .split('/')
    .map((part) => part.trim())
    .filter(Boolean)

  const normalizedParts = parts.length ? parts : [folder.name.trim()]
  const leafName = normalizedParts[normalizedParts.length - 1] || folder.name.trim()

  return {
    folder,
    depth: Math.max(0, normalizedParts.length - 1),
    leafName,
    pathPrefix: normalizedParts.slice(0, -1).join(' / '),
    pathParts: normalizedParts
  }
}

export function orderCommunicationFolderDisplayRows(folders: ReadonlyArray<CommunicationFolder>): CommunicationFolderDisplayRow[] {
  return folders
    .map((folder) => deriveCommunicationFolderDisplayRow(folder))
    .sort(compareCommunicationFolderRows)
}

export function createChildFolderDraft(folder: CommunicationFolder): {
  parentPath: string
  sortOrder: number
} {
  return {
    parentPath: folder.name,
    sortOrder: folder.sort_order
  }
}

export function mailFolderHierarchyDeleteImpact(
  folders: ReadonlyArray<CommunicationFolder>,
  folderId: string
): CommunicationFolderHierarchyDeleteImpact {
  const rows = orderCommunicationFolderDisplayRows(folders)
  const target = rows.find((row) => row.folder.folder_id === folderId)
  if (!target) {
    return {
      descendantCount: 0,
      descendantLeafNames: []
    }
  }

  const descendants = rows.filter((row) =>
    row.folder.folder_id !== folderId && isDescendantPath(target.pathParts, row.pathParts)
  )

  return {
    descendantCount: descendants.length,
    descendantLeafNames: descendants.slice(0, 3).map((row) => row.leafName)
  }
}

function compareCommunicationFolderRows(left: CommunicationFolderDisplayRow, right: CommunicationFolderDisplayRow): number {
  if (left.folder.sort_order !== right.folder.sort_order) {
    return left.folder.sort_order - right.folder.sort_order
  }

  const segmentCount = Math.min(left.pathParts.length, right.pathParts.length)
  for (let index = 0; index < segmentCount; index += 1) {
    const comparison = left.pathParts[index].localeCompare(right.pathParts[index], undefined, {
      sensitivity: 'base'
    })
    if (comparison !== 0) return comparison
  }

  if (left.pathParts.length !== right.pathParts.length) {
    return left.pathParts.length - right.pathParts.length
  }

  return left.folder.folder_id.localeCompare(right.folder.folder_id)
}

function isDescendantPath(parentPathParts: string[], candidatePathParts: string[]): boolean {
  if (candidatePathParts.length <= parentPathParts.length) return false
  return parentPathParts.every((part, index) => (
    part.localeCompare(candidatePathParts[index] ?? '', undefined, { sensitivity: 'base' }) === 0
  ))
}
```

### `frontend/src/domains/communications/components/outboxStatus.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/components/outboxStatus.test.ts`
- Size bytes / Размер в байтах: `3653`
- Included characters / Включено символов: `3653`
- Truncated / Обрезано: `no`

```typescript
import { describe, expect, it } from 'vitest'
import type { CommunicationOutboxItem } from '../types/communications'
import {
  outboxStatusPresentation,
  visibleOutboxStatusItems
} from './outboxStatus'

function outboxItem(overrides: Partial<CommunicationOutboxItem> = {}): CommunicationOutboxItem {
  return {
    outbox_id: 'outbox-1',
    account_id: 'account-1',
    draft_id: null,
    to_recipients: ['reader@example.com'],
    cc_recipients: [],
    bcc_recipients: [],
    subject: 'Quarterly update',
    body_text: 'Body',
    body_html: null,
    status: 'sent',
    scheduled_send_at: null,
    undo_deadline_at: null,
    send_attempts: 1,
    claimed_at: null,
    sent_at: '2026-06-15T09:00:00Z',
    provider_message_id: 'provider-message-1',
    last_error: null,
    metadata: {},
    created_at: '2026-06-15T08:59:00Z',
    updated_at: '2026-06-15T09:00:00Z',
    ...overrides
  }
}

describe('outbox status presentation', () => {
  it('prioritizes latest read receipt evidence for sent outbox items', () => {
    const item = outboxItem({
      metadata: {
        latest_read_receipt: {
          receipt_kind: 'read',
          read_at: '2026-06-15T09:10:00Z',
          source_kind: 'mdn'
        }
      }
    })

    expect(outboxStatusPresentation(item, new Date('2026-06-15T09:12:00Z'))).toMatchObject({
      title: 'Read',
      detail: 'Read at Jun 15, 09:10',
      tone: 'success',
      icon: 'tabler:mail-check'
    })
  })

  it('shows provider delivery failure evidence without exposing diagnostics', () => {
    const item = outboxItem({
      metadata: {
        delivery_status: {
          delivery_status: 'failed',
          smtp_status: '5.1.1',
          source_kind: 'dsn',
          diagnostic_code: 'smtp; private mailbox detail'
        }
      }
    })

    const presentation = outboxStatusPresentation(item, new Date('2026-06-15T09:12:00Z'))

    expect(presentation).toMatchObject({
      title: 'Delivery failed',
      detail: 'Provider reported SMTP 5.1.1',
      tone: 'danger',
      icon: 'tabler:alert-triangle'
    })
    expect(presentation.detail).not.toContain('private mailbox detail')
  })

  it('shows undo and retry timing for queued outbox records', () => {
    expect(outboxStatusPresentation(outboxItem({
      status: 'queued',
      undo_deadline_at: '2026-06-15T09:05:00Z',
      sent_at: null,
      provider_message_id: null
    }), new Date('2026-06-15T09:01:00Z'))).toMatchObject({
      title: 'Undo available',
      canUndo: true,
      tone: 'warning'
    })

    expect(outboxStatusPresentation(outboxItem({
      status: 'scheduled',
      scheduled_send_at: '2026-06-15T09:30:00Z',
      send_attempts: 2,
      last_error: 'SMTP send failed',
      sent_at: null,
      provider_message_id: null
    }), new Date('2026-06-15T09:01:00Z'))).toMatchObject({
      title: 'Retry scheduled',
      detail: 'Retry at Jun 15, 09:30',
      canUndo: false,
      tone: 'warning'
    })
  })

  it('filters out terminal sent items without fresh delivery evidence from the compact strip', () => {
    const items = visibleOutboxStatusItems([
      outboxItem({ outbox_id: 'sent-plain' }),
      outboxItem({
        outbox_id: 'sent-read',
        metadata: {
          latest_read_receipt: {
            receipt_kind: 'read',
            read_at: '2026-06-15T09:10:00Z',
            source_kind: 'mdn'
          }
        }
      }),
      outboxItem({
        outbox_id: 'queued',
        status: 'queued',
        sent_at: null,
        provider_message_id: null
      })
    ], 4)

    expect(items.map((item) => item.outbox_id)).toEqual(['queued', 'sent-read'])
  })
})
```
