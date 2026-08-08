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

- Chunk ID / ID чанка: `146-source-frontend-part-006`
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

### `frontend/src/domains/communications/forms/savedSearchForm.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/forms/savedSearchForm.test.ts`
- Size bytes / Размер в байтах: `10173`
- Included characters / Включено символов: `10173`
- Truncated / Обрезано: `no`

```typescript
import { describe, expect, it } from 'vitest'
import {
  composeSavedSearchRuleTreeQuery,
  createSavedSearchRuleCondition,
  createSavedSearchRuleGroup,
  composeSavedSearchQuery,
  flattenSavedSearchRuleTree,
  normalizeSavedSearchBuilderState,
  parseSavedSearchQuery,
  resolveSavedSearchEffectiveQuery,
  savedSearchDeleteDialogCopy,
  savedSearchChannelOptions,
  savedSearchFilterChips,
  savedSearchFormSchema,
  savedSearchFormToInput,
  savedSearchMessageCountLabel,
  savedSearchPresetOptions,
  validateSavedSearchRuleTree,
  validateSavedSearchRules
} from './savedSearchForm'

describe('saved search form', () => {
  it('normalizes form values into a saved-search input payload', () => {
    const values = savedSearchFormSchema.parse({
      name: '  Finance invoices  ',
      description: '  Quarterly invoice filter  ',
      query: '  invoice due  ',
      workflow_state: 'needs_action',
      local_state: 'active',
      channel_kind: '  email  ',
      is_smart_folder: false
    })

    expect(savedSearchFormToInput(values, 'account-1')).toEqual({
      name: 'Finance invoices',
      description: 'Quarterly invoice filter',
      account_id: 'account-1',
      query: 'invoice due',
      workflow_state: 'needs_action',
      local_state: 'active',
      channel_kind: 'email',
      is_smart_folder: false
    })
  })

  it('allows smart folders with empty query and rejects empty names', () => {
    expect(() =>
      savedSearchFormSchema.parse({
        name: ' ',
        description: '',
        query: '',
        workflow_state: null,
        local_state: 'active',
        channel_kind: '',
        is_smart_folder: true
      })
    ).toThrow()

    const values = savedSearchFormSchema.parse({
      name: 'Needs reply',
      description: '',
      query: '',
      workflow_state: 'needs_action',
      local_state: 'active',
      channel_kind: '',
      is_smart_folder: true
    })

    expect(savedSearchFormToInput(values, null)).toMatchObject({
      name: 'Needs reply',
      account_id: null,
      query: '',
      workflow_state: 'needs_action',
      local_state: 'active',
      channel_kind: null,
      is_smart_folder: true
    })
  })

  it('offers Telegram as a first-class saved-search channel kind', () => {
    expect(savedSearchChannelOptions).toEqual(
      expect.arrayContaining([
        { label: 'Telegram', value: 'telegram' }
      ])
    )
  })

  it('builds delete confirmation copy for saved searches and smart folders', () => {
    expect(savedSearchDeleteDialogCopy({ name: 'Finance', is_smart_folder: false })).toEqual({
      title: 'Delete saved search',
      message: 'Delete the saved search "Finance"? This does not delete messages.',
      confirmLabel: 'Delete'
    })

    expect(savedSearchDeleteDialogCopy({ name: 'Needs reply', is_smart_folder: true })).toEqual({
      title: 'Delete smart folder',
      message: 'Delete the smart folder "Needs reply"? This does not delete messages.',
      confirmLabel: 'Delete'
    })
  })

  it('formats saved-search message counts for compact strip badges', () => {
    expect(savedSearchMessageCountLabel({ message_count: 0 })).toBe('0')
    expect(savedSearchMessageCountLabel({ message_count: 42 })).toBe('42')
    expect(savedSearchMessageCountLabel({ message_count: 1200 })).toBe('1200')
  })

  it('summarizes active saved-search filters as rule chips', () => {
    const values = savedSearchFormSchema.parse({
      name: 'Escalations',
      description: '',
      query: ' renewal ',
      workflow_state: 'waiting',
      local_state: 'all',
      channel_kind: ' email ',
      is_smart_folder: true
    })

    expect(savedSearchFilterChips(values)).toEqual([
      { label: 'Text', value: 'renewal' },
      { label: 'Workflow', value: 'Waiting' },
      { label: 'Scope', value: 'All' },
      { label: 'Channel', value: 'email' },
      { label: 'Mode', value: 'Smart folder' }
    ])
  })

  it('does not throw when rendering filter chips for an unsaved invalid draft', () => {
    expect(() => savedSearchFilterChips({
      name: '',
      description: '',
      query: ' invoice ',
      workflow_state: null,
      local_state: 'active',
      channel_kind: ' email ',
      is_smart_folder: false,
      match_mode: 'all'
    }, [
      { field: 'sender', operator: ':', value: 'alex' }
    ])).not.toThrow()

    expect(savedSearchFilterChips({
      name: '',
      description: '',
      query: ' invoice ',
      workflow_state: null,
      local_state: 'active',
      channel_kind: ' email ',
      is_smart_folder: false,
      match_mode: 'all'
    }, [
      { field: 'sender', operator: ':', value: 'alex' }
    ])).toEqual([
      { label: 'Text', value: 'invoice' },
      { label: 'Sender contains', value: 'alex' },
      { label: 'Channel', value: 'email' },
      { label: 'Mode', value: 'Saved search' }
    ])
  })

  it('provides safe Mail-only presets for smart filters', () => {
    expect(savedSearchPresetOptions).toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          label: 'Needs action',
          values: expect.objectContaining({
            workflow_state: 'needs_action',
            local_state: 'active',
            is_smart_folder: true
          })
        }),
        expect.objectContaining({
          label: 'Spam review',
          values: expect.objectContaining({
            workflow_state: 'spam',
            local_state: 'active',
            is_smart_folder: true
          })
        })
      ])
    )
  })

  it('parses search rules from query text', () => {
    expect(parseSavedSearchQuery('subject:"invoice due" from:alex body:payment all==todo')).toMatchObject({
      plainQuery: '',
      rules: [
        { field: 'subject', operator: ':', value: 'invoice due' },
        { field: 'sender', operator: ':', value: 'alex' },
        { field: 'body', operator: ':', value: 'payment' },
        { field: 'all', operator: '=', value: 'todo' }
      ],
      matchMode: 'all'
    })
  })

  it('builds a saved-search query from plain text and rules', () => {
    expect(composeSavedSearchQuery('invoice', [
      { field: 'sender', operator: ':', value: 'alex' },
      { field: 'subject', operator: '=', value: 'Quarterly Report' }
    ])).toBe('invoice sender:alex subject="Quarterly Report"')
  })

  it('supports any-match query composition and filter chips', () => {
    const values = savedSearchFormSchema.parse({
      name: 'Flexible',
      description: '',
      query: 'invoice',
      workflow_state: null,
      local_state: 'active',
      channel_kind: '',
      is_smart_folder: false,
      match_mode: 'any'
    })

    expect(composeSavedSearchQuery('invoice', [
      { field: 'sender', operator: ':', value: 'alex' }
    ], 'any')).toBe('mode:any invoice sender:alex')

    expect(savedSearchFilterChips(values, [
      { field: 'sender', operator: ':', value: 'alex' }
    ])).toEqual([
      { label: 'Text', value: 'invoice' },
      { label: 'Sender contains', value: 'alex' },
      { label: 'Match', value: 'Any condition' },
      { label: 'Mode', value: 'Saved search' }
    ])
  })

  it('normalizes structured query tokens into builder state without duplicating rules', () => {
    expect(normalizeSavedSearchBuilderState(
      'mode:any invoice from:alex subject:"Quarterly Report"',
      [
        { field: 'sender', operator: ':', value: 'alex' },
        { field: 'body', operator: ':', value: 'payment' }
      ],
      'all'
    )).toMatchObject({
      plainQuery: 'invoice',
      rules: [
        { field: 'sender', operator: ':', value: 'alex' },
        { field: 'subject', operator: ':', value: 'Quarterly Report' },
        { field: 'body', operator: ':', value: 'payment' }
      ],
      matchMode: 'any'
    })
  })

  it('parses nested rule groups from explicit boolean expressions', () => {
    const parsed = parseSavedSearchQuery('(subject:quarterly OR body:invoice) AND sender:alex')

    expect(parsed.plainQuery).toBe('')
    expect(parsed.matchMode).toBe('all')
    expect(flattenSavedSearchRuleTree(parsed.tree)).toEqual([
      { field: 'subject', operator: ':', value: 'quarterly' },
      { field: 'body', operator: ':', value: 'invoice' },
      { field: 'sender', operator: ':', value: 'alex' }
    ])
  })

  it('composes nested rule groups into an explicit preview query', () => {
    const tree = createSavedSearchRuleGroup('all', [
      createSavedSearchRuleGroup('any', [
        createSavedSearchRuleCondition({ field: 'subject', operator: ':', value: 'quarterly' }),
        createSavedSearchRuleCondition({ field: 'body', operator: ':', value: 'invoice' })
      ]),
      createSavedSearchRuleCondition({ field: 'sender', operator: ':', value: 'alex' })
    ])

    expect(composeSavedSearchRuleTreeQuery('', tree)).toBe('((subject:quarterly OR body:invoice) AND sender:alex)')
  })

  it('resolves the effective saved-search query from normalized builder state', () => {
    expect(resolveSavedSearchEffectiveQuery(
      'mode:any invoice from:alex',
      [{ field: 'subject', operator: ':', value: 'Quarterly Report' }],
      'all'
    )).toBe('mode:any invoice sender:alex subject:"Quarterly Report"')
  })

  it('validates incomplete and duplicate builder rules before save', () => {
    expect(validateSavedSearchRules([
      { field: 'subject', operator: ':', value: '' }
    ])).toEqual({
      isValid: false,
      message: 'Complete or remove the empty rule before saving'
    })

    expect(validateSavedSearchRules([
      { field: 'sender', operator: ':', value: 'alex' },
      { field: 'sender', operator: ':', value: ' Alex ' }
    ])).toEqual({
      isValid: false,
      message: 'Remove duplicate rules before saving'
    })

    expect(validateSavedSearchRules([
      { field: 'sender', operator: ':', value: 'alex' },
      { field: 'subject', operator: ':', value: 'Quarterly Report' }
    ])).toEqual({
      isValid: true,
      message: ''
    })

    expect(validateSavedSearchRuleTree(createSavedSearchRuleGroup('all', []))).toEqual({
      isValid: false,
      message: 'Add at least one rule or group before saving'
    })
  })
})
```

### `frontend/src/domains/communications/forms/savedSearchForm.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/forms/savedSearchForm.ts`
- Size bytes / Размер в байтах: `6921`
- Included characters / Включено символов: `6921`
- Truncated / Обрезано: `no`

```typescript
import { toTypedSchema } from '@vee-validate/zod'
import { z } from 'zod'
import type { LocalMessageState, WorkflowState } from '../types/communications'
import type { CommunicationSavedSearch, SavedSearchInput } from '../types/savedSearches'
import type { SavedSearchMatchMode, SavedSearchRule, SavedSearchRuleField, SavedSearchRuleOperator } from './savedSearchRuleTree'
import {
  savedSearchLocalStateLabels,
  savedSearchLocalStates,
  savedSearchMatchModeLabels,
  savedSearchWorkflowStateLabels,
  savedSearchWorkflowStates
} from './savedSearchFormOptions'

export {
  savedSearchLocalStateOptions,
  savedSearchMatchModeLabels,
  savedSearchWorkflowOptions
} from './savedSearchFormOptions'
export {
  composeSavedSearchQuery,
  composeSavedSearchRuleTreeQuery,
  createSavedSearchRuleCondition,
  createSavedSearchRuleGroup,
  flattenSavedSearchRuleTree,
  normalizeSavedSearchBuilderState,
  parseSavedSearchQuery,
  resolveSavedSearchEffectiveQuery,
  tokenizeSavedSearchQuery,
  validateSavedSearchRules,
  validateSavedSearchRuleTree
} from './savedSearchRuleTree'
export type {
  SavedSearchBuilderState,
  SavedSearchMatchMode,
  SavedSearchParsedQuery,
  SavedSearchRule,
  SavedSearchRuleCondition,
  SavedSearchRuleField,
  SavedSearchRuleGroup,
  SavedSearchRuleNode,
  SavedSearchRuleOperator,
  SavedSearchRuleValidation
} from './savedSearchRuleTree'
export type SavedSearchFormValues = z.infer<typeof savedSearchFormSchema>
export type SavedSearchDeleteDialogCopy = {
  title: string
  message: string
  confirmLabel: string
}
export type SavedSearchOption<T extends string | null = string> = {
  label: string
  value: T
}
export type SavedSearchFilterChip = {
  label: string
  value: string
}
export type SavedSearchPresetOption = {
  label: string
  values: Pick<SavedSearchFormValues, 'query' | 'workflow_state' | 'local_state' | 'channel_kind' | 'is_smart_folder'>
}

export const savedSearchFormSchema = z.object({
  name: z.string().trim().min(1, 'Name is required').max(120, 'Name is too long'),
  description: z.string().trim().max(500, 'Description is too long'),
  query: z.string().trim().max(500, 'Search query is too long'),
  workflow_state: z.enum(savedSearchWorkflowStates).nullable(),
  local_state: z.enum(savedSearchLocalStates),
  channel_kind: z.string().trim().max(80, 'Channel is too long'),
  is_smart_folder: z.boolean(),
  match_mode: z.enum(['all', 'any']).default('all')
})

export const savedSearchVeeValidationSchema = toTypedSchema(savedSearchFormSchema)
export const savedSearchChannelOptions: Array<SavedSearchOption> = [
  { label: 'Email', value: 'email' },
  { label: 'Telegram', value: 'telegram' },
  { label: 'Any channel', value: '' }
]
export const savedSearchMatchModeOptions: Array<SavedSearchOption<SavedSearchMatchMode>> = [
  { label: savedSearchMatchModeLabels.all, value: 'all' },
  { label: savedSearchMatchModeLabels.any, value: 'any' }
]
export const savedSearchPresetOptions: SavedSearchPresetOption[] = [
  {
    label: 'Needs action',
    values: {
      query: '',
      workflow_state: 'needs_action',
      local_state: 'active',
      channel_kind: 'email',
      is_smart_folder: true
    }
  },
  {
    label: 'Waiting',
    values: {
      query: '',
      workflow_state: 'waiting',
      local_state: 'active',
      channel_kind: 'email',
      is_smart_folder: true
    }
  },
  {
    label: 'Spam review',
    values: {
      query: '',
      workflow_state: 'spam',
      local_state: 'active',
      channel_kind: 'email',
      is_smart_folder: true
    }
  }
]

const savedSearchRuleFieldLabels: Record<SavedSearchRuleField, string> = {
  subject: 'Subject',
  body: 'Body',
  sender: 'Sender',
  all: 'All'
}
const savedSearchRuleOperatorLabels: Record<SavedSearchRuleOperator, string> = {
  ':': 'contains',
  '=': 'equals'
}

export function savedSearchFormDefaults(
  savedSearch?: CommunicationSavedSearch | null,
  isSmartFolder = false
): SavedSearchFormValues {
  return {
    name: savedSearch?.name ?? '',
    description: savedSearch?.description ?? '',
    query: savedSearch?.query ?? '',
    workflow_state: savedSearch?.workflow_state ?? null,
    local_state: savedSearch?.local_state ?? 'active',
    channel_kind: savedSearch?.channel_kind ?? '',
    is_smart_folder: savedSearch?.is_smart_folder ?? isSmartFolder,
    match_mode: 'all'
  }
}

export function savedSearchFormToInput(
  values: SavedSearchFormValues,
  accountId: string | null
): SavedSearchInput {
  const parsed = savedSearchFormSchema.parse(values)
  return {
    name: parsed.name,
    description: parsed.description || null,
    account_id: accountId?.trim() || null,
    query: parsed.query,
    workflow_state: parsed.workflow_state as WorkflowState | null,
    local_state: parsed.local_state as LocalMessageState,
    channel_kind: parsed.channel_kind || null,
    is_smart_folder: parsed.is_smart_folder
  }
}

export function savedSearchFilterChips(
  values: SavedSearchFormValues,
  rules: SavedSearchRule[] = []
): SavedSearchFilterChip[] {
  const parsedResult = savedSearchFormSchema.safeParse(values)
  const parsed = parsedResult.success
    ? parsedResult.data
    : {
        ...values,
        name: values.name.trim(),
        description: values.description.trim(),
        query: values.query.trim(),
        channel_kind: values.channel_kind.trim()
      }
  const chips: SavedSearchFilterChip[] = []
  const query = parsed.query.trim()
  const channel = parsed.channel_kind.trim()

  if (query) chips.push({ label: 'Text', value: query })
  for (const rule of rules) {
    if (!rule.value.trim()) continue
    chips.push({
      label: `${savedSearchRuleFieldLabels[rule.field]} ${savedSearchRuleOperatorLabels[rule.operator]}`,
      value: rule.value
    })
  }
  if (parsed.workflow_state) {
    chips.push({ label: 'Workflow', value: savedSearchWorkflowStateLabels[parsed.workflow_state] })
  }
  if (parsed.local_state !== 'active') {
    chips.push({ label: 'Scope', value: savedSearchLocalStateLabels[parsed.local_state] })
  }
  if (parsed.match_mode === 'any') {
    chips.push({ label: 'Match', value: savedSearchMatchModeLabels.any })
  }
  if (channel) chips.push({ label: 'Channel', value: channel })
  chips.push({ label: 'Mode', value: parsed.is_smart_folder ? 'Smart folder' : 'Saved search' })

  return chips
}

export function savedSearchDeleteDialogCopy(
  savedSearch: Pick<CommunicationSavedSearch, 'name' | 'is_smart_folder'>
): SavedSearchDeleteDialogCopy {
  const kind = savedSearch.is_smart_folder ? 'smart folder' : 'saved search'
  return {
    title: `Delete ${kind}`,
    message: `Delete the ${kind} "${savedSearch.name}"? This does not delete messages.`,
    confirmLabel: 'Delete'
  }
}

export function savedSearchMessageCountLabel(
  savedSearch: Pick<CommunicationSavedSearch, 'message_count'>
): string {
  return String(Math.max(0, Math.trunc(savedSearch.message_count)))
}
```

### `frontend/src/domains/communications/forms/savedSearchFormOptions.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/forms/savedSearchFormOptions.ts`
- Size bytes / Размер в байтах: `1309`
- Included characters / Включено символов: `1309`
- Truncated / Обрезано: `no`

```typescript
import type { LocalMessageState, WorkflowState } from '../types/communications'

export const savedSearchWorkflowStates = [
  'new',
  'reviewed',
  'needs_action',
  'waiting',
  'done',
  'archived',
  'muted',
  'spam'
] as const

export const savedSearchLocalStates = ['active', 'trash', 'all'] as const

export const savedSearchWorkflowStateLabels: Record<(typeof savedSearchWorkflowStates)[number], string> = {
  new: 'New',
  reviewed: 'Reviewed',
  needs_action: 'Needs action',
  waiting: 'Waiting',
  done: 'Done',
  archived: 'Archived',
  muted: 'Muted',
  spam: 'Spam'
}

export const savedSearchLocalStateLabels: Record<(typeof savedSearchLocalStates)[number], string> = {
  active: 'Active',
  trash: 'Trash',
  all: 'All'
}

export const savedSearchMatchModeLabels: Record<'all' | 'any', string> = {
  all: 'All conditions',
  any: 'Any condition'
}

export const savedSearchWorkflowOptions: Array<{ label: string; value: WorkflowState | null }> = [
  { label: 'Any', value: null },
  ...savedSearchWorkflowStates.map((value) => ({ label: savedSearchWorkflowStateLabels[value], value }))
]

export const savedSearchLocalStateOptions: Array<{ label: string; value: LocalMessageState }> =
  savedSearchLocalStates.map((value) => ({
    label: savedSearchLocalStateLabels[value],
    value
  }))
```

### `frontend/src/domains/communications/forms/savedSearchRuleTree.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/forms/savedSearchRuleTree.ts`
- Size bytes / Размер в байтах: `14790`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```typescript
export type SavedSearchMatchMode = 'all' | 'any'
export type SavedSearchRuleValidation = {
  isValid: boolean
  message: string
}
export type SavedSearchRuleField = 'subject' | 'body' | 'sender' | 'all'
export type SavedSearchRuleOperator = ':' | '='
export type SavedSearchRule = {
  field: SavedSearchRuleField
  operator: SavedSearchRuleOperator
  value: string
}
export type SavedSearchRuleCondition = SavedSearchRule & {
  id: string
  kind: 'rule'
}
export type SavedSearchRuleGroup = {
  id: string
  kind: 'group'
  matchMode: SavedSearchMatchMode
  children: SavedSearchRuleNode[]
}
export type SavedSearchRuleNode = SavedSearchRuleCondition | SavedSearchRuleGroup
export type SavedSearchParsedQuery = {
  plainQuery: string
  rules: SavedSearchRule[]
  matchMode: SavedSearchMatchMode
  tree: SavedSearchRuleGroup
}
export type SavedSearchBuilderState = SavedSearchParsedQuery

export function tokenizeSavedSearchQuery(rawQuery: string): string[] {
  const terms: string[] = []
  let current = ''
  let inQuotes = false
  let quote: string | null = null

  for (const symbol of rawQuery) {
    if ((symbol === '"' || symbol === "'") && (!inQuotes || quote === symbol)) {
      inQuotes = !inQuotes
      quote = inQuotes ? symbol : null
      current += symbol
      continue
    }

    if (symbol.trim() === '' && !inQuotes) {
      if (current.trim()) {
        terms.push(current.trim())
      }
      current = ''
      continue
    }

    current += symbol
  }

  if (current.trim()) terms.push(current.trim())
  return terms
}

export function parseSavedSearchQuery(rawQuery: string): SavedSearchParsedQuery {
  const normalized = rawQuery.trim()
  if (!normalized) {
    return {
      plainQuery: '',
      rules: [],
      matchMode: 'all',
      tree: createSavedSearchRuleGroup('all', [])
    }
  }

  const explicitTree = parseExplicitSavedSearchTree(normalized)
  if (explicitTree) {
    return {
      plainQuery: '',
      rules: flattenSavedSearchRuleTree(explicitTree),
      matchMode: explicitTree.matchMode,
      tree: explicitTree
    }
  }

  const rules: SavedSearchRule[] = []
  const plainQueryTerms: string[] = []
  let matchMode: SavedSearchMatchMode = 'all'

  for (const token of tokenizeSavedSearchQuery(normalized)) {
    const parsedMode = parseSavedSearchMatchMode(token)
    if (parsedMode) {
      matchMode = parsedMode
      continue
    }

    const rule = parseSavedSearchRuleToken(token)
    if (rule) {
      rules.push(rule)
      continue
    }
    plainQueryTerms.push(token)
  }

  return {
    plainQuery: plainQueryTerms.join(' '),
    rules,
    matchMode,
    tree: createSavedSearchRuleGroup(matchMode, rules.map((rule) => createSavedSearchRuleCondition(rule)))
  }
}

export function composeSavedSearchQuery(
  plainQuery: string,
  rules: ReadonlyArray<SavedSearchRule>,
  matchMode: SavedSearchMatchMode = 'all'
): string {
  const ruleTokens = rules
    .map((rule) => {
      const trimmedValue = rule.value.trim()
      if (!trimmedValue) return ''
      return `${rule.field}${rule.operator}${formatSavedSearchRuleValue(trimmedValue)}`
    })
    .filter(Boolean)

  const merged: string[] = []
  if (plainQuery.trim()) merged.push(plainQuery.trim())
  if (ruleTokens.length) merged.push(...ruleTokens)
  if (matchMode === 'any' && merged.length > 0) {
    merged.unshift('mode:any')
  }

  return merged.join(' ')
}

export function normalizeSavedSearchBuilderState(
  rawQuery: string,
  existingRules: ReadonlyArray<SavedSearchRule> = [],
  currentMatchMode: SavedSearchMatchMode = 'all'
): SavedSearchBuilderState {
  const parsed = parseSavedSearchQuery(rawQuery)
  const mergedRules = mergeSavedSearchRules(parsed.rules, existingRules)
  const matchMode = parsed.matchMode === 'any' ? 'any' : currentMatchMode

  return {
    plainQuery: parsed.plainQuery,
    rules: mergedRules,
    matchMode,
    tree: parsed.tree.kind === 'group'
      ? parsed.tree
      : createSavedSearchRuleGroup(matchMode, mergedRules.map((rule) => createSavedSearchRuleCondition(rule)))
  }
}

export function resolveSavedSearchEffectiveQuery(
  rawQuery: string,
  existingRules: ReadonlyArray<SavedSearchRule> = [],
  currentMatchMode: SavedSearchMatchMode = 'all'
): string {
  const normalized = normalizeSavedSearchBuilderState(rawQuery, existingRules, currentMatchMode)
  return composeSavedSearchQuery(normalized.plainQuery, normalized.rules, normalized.matchMode)
}

export function validateSavedSearchRules(
  rules: ReadonlyArray<SavedSearchRule>
): SavedSearchRuleValidation {
  const normalizedRules = rules
    .map((rule) => ({
      field: rule.field,
      operator: rule.operator,
      value: rule.value.trim()
    }))

  const incompleteCount = normalizedRules.filter((rule) => !rule.value).length
  if (incompleteCount > 0) {
    return {
      isValid: false,
      message: incompleteCount === 1
        ? 'Complete or remove the empty rule before saving'
        : 'Complete or remove the empty rules before saving'
    }
  }

  const seen = new Set<string>()
  for (const rule of normalizedRules) {
    const signature = `${rule.field}|${rule.operator}|${rule.value.toLowerCase()}`
    if (seen.has(signature)) {
      return {
        isValid: false,
        message: 'Remove duplicate rules before saving'
      }
    }
    seen.add(signature)
  }

  return {
    isValid: true,
    message: ''
  }
}

export function validateSavedSearchRuleTree(
  group: SavedSearchRuleGroup
): SavedSearchRuleValidation {
  if (group.children.length === 0) {
    return {
      isValid: false,
      message: 'Add at least one rule or group before saving'
    }
  }

  const flattened = flattenSavedSearchRuleTree(group)
  const ruleValidation = validateSavedSearchRules(flattened)
  if (!ruleValidation.isValid) return ruleValidation

  for (const child of group.children) {
    if (child.kind === 'group') {
      const nestedValidation = validateSavedSearchRuleTree(child)
      if (!nestedValidation.isValid) return nestedValidation
    }
  }

  return {
    isValid: true,
    message: ''
  }
}

export function createSavedSearchRuleCondition(
  rule: Partial<SavedSearchRule> = {}
): SavedSearchRuleCondition {
  return {
    id: savedSearchBuilderId('rule'),
    kind: 'rule',
    field: rule.field ?? 'subject',
    operator: rule.operator ?? ':',
    value: rule.value ?? ''
  }
}

export function createSavedSearchRuleGroup(
  matchMode: SavedSearchMatchMode = 'all',
  children: SavedSearchRuleNode[] = []
): SavedSearchRuleGroup {
  return {
    id: savedSearchBuilderId('group'),
    kind: 'group',
    matchMode,
    children
  }
}

export function flattenSavedSearchRuleTree(
  group: SavedSearchRuleGroup
): SavedSearchRule[] {
  return group.children.flatMap((child) => {
    if (child.kind === 'group') return flattenSavedSearchRuleTree(child)
    return [{ field: child.field, operator: child.operator, value: child.value }]
  })
}

export function composeSavedSearchRuleTreeQuery(
  plainQuery: string,
  group: SavedSearchRuleGroup
): string {
  if (group.children.length === 0) {
    return plainQuery.trim()
  }

  const hasNestedGroups = group.children.some((child) => child.kind === 'group')
  if (!hasNestedGroups && group.matchMode === 'all') {
    return composeSavedSearchQuery(plainQuery, flattenSavedSearchRuleTree(group), 'all')
  }

  const plainTerms = plainQuery.trim()
    ? tokenizeSavedSearchQuery(plainQuery).map((term) =>
        createSavedSearchRuleCondition({ field: 'all', operator: ':', value: parseSavedSearchRuleValue(term) })
      )
    : []
  const rootChildren = group.matchMode === 'any'
    ? [...plainTerms, ...group.children]
    : group.children
  const expression = formatSavedSearchRuleGroupExpression({
    ...group,
    children: rootChildren
  })
  if (group.matchMode === 'all' && plainQuery.trim()) {
    return `${plainQuery.trim()} ${expression}`.trim()
  }
  return expression
}

function parseSavedSearchMatchMode(token: string): SavedSearchMatchMode | null {
  const [rawField, rawValue] = token.split(':', 2)
  if (rawValue === undefined) return null
  if (rawField.trim().toLowerCase() !== 'mode') return null

  const value = rawValue.trim().toLowerCase()
  if (value === 'all' || value === 'any') return value
  return null
}

function parseSavedSearchRuleToken(token: string): SavedSearchRule | null {
  const operators: Array<SavedSearchRuleOperator | '=='> = ['==', '=', ':']

  for (const operator of operators) {
    const index = token.indexOf(operator)
    if (index <= 0) continue

    const rawField = token.slice(0, index)
    const normalizedOperator: SavedSearchRuleOperator = operator === '==' ? '=' : operator
    const rawValue = token.slice(index + operator.length)

    const field = parseSavedSearchRuleField(rawField)
    if (!field || !rawValue.trim()) return null

    const value = parseSavedSearchRuleValue(rawValue)
    if (!value) return null

    return { field, operator: normalizedOperator, value }
  }

  return null
}

function parseSavedSearchRuleField(value: string): SavedSearchRuleField | null {
  const normalized = value.trim().toLowerCase()
  if (normalized === 'from') return 'sender'
  if (normalized === 'subject' || normalized === 'body' || normalized === 'sender' || normalized === 'all') {
    return normalized as SavedSearchRuleField
  }
  return null
}

function parseSavedSearchRuleValue(rawValue: string): string {
  const value = rawValue.trim()
  if (value.length < 2) return value

  const isDouble = value.startsWith('"') && value.endsWith('"')
  const isSingle = value.startsWith("'") && value.endsWith("'")
  if (!isDouble && !isSingle) return value

  return value.slice(1, -1)
}

function formatSavedSearchRuleValue(value: string): string {
  const trimmed = value.trim()
  if (!trimmed) return ''
  const needsQuote = trimmed.includes(' ') || trimmed.includes('"') || trimmed.includes("'")
  if (!needsQuote) return trimmed
  return `"${trimmed.replaceAll('"', '\\"')}"`
}

function mergeSavedSearchRules(
  parsedRules: ReadonlyArray<SavedSearchRule>,
  existingRules: ReadonlyArray<SavedSearchRule>
): SavedSearchRule[] {
  const merged: SavedSearchRule[] = []
  const seen = new Set<string>()

  for (const rule of [...parsedRules, ...existingRules]) {
    const normalized = normalizeSavedSearchRule(rule)
    if (!normalized) continue
    const signature = `${normalized.field}|${normalized.operator}|${normalized.value.toLowerCase()}`
    if (seen.has(signature)) continue
    seen.add(signature)
    merged.push(normalized)
  }

  return merged
}

function normalizeSavedSearchRule(rule: SavedSearchRule): SavedSearchRule | null {
  const value = rule.value.trim()
  if (!value) return null

  return {
    field: rule.field,
    operator: rule.operator,
    value
  }
}

function parseExplicitSavedSearchTree(rawQuery: string): SavedSearchRuleGroup | null {
  const tokens = tokenizeSavedSearchExpression(rawQuery)
  if (!tokens.some((token) => token === '(' || token === ')' || token === 'AND' || token === 'OR')) {
    return null
  }
  const parser = createSavedSearchExpressionParser(tokens)
  const expression = parser.parseExpression()
  if (!expression || parser.hasRemainingTokens()) return null
  return expression
}

function tokenizeSavedSearchExpression(rawQuery: string): string[] {
  const tokens: string[] = []
  let current = ''
  let inQuotes = false
  let quote: string | null = null

  const pushCurrent = () => {
    const normalized = current.trim()
    if (normalized) tokens.push(normalized)
    current = ''
  }

  for (const symbol of rawQuery) {
    if ((symbol === '"' || symbol === "'") && (!inQuotes || quote === symbol)) {
      inQuotes = !inQuotes
      quote = inQuotes ? symbol : null
      current += symbol
      continue
    }

    if (!inQuotes && (symbol === '(' || symbol === ')')) {
      pushCurrent()
      tokens.push(symbol)
      continue
    }

    if (!inQuotes && symbol.trim() === '') {
      pushCurrent()
      continue
    }

    current += symb
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `frontend/src/domains/communications/forms/templateForm.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/forms/templateForm.test.ts`
- Size bytes / Размер в байтах: `5464`
- Included characters / Включено символов: `5464`
- Truncated / Обрезано: `no`

```typescript
import { describe, expect, it } from 'vitest'
import {
  defaultTemplateVariableValue,
  extractTemplateVariables,
  missingTemplateVariables,
  parseTemplateMailMergePreviewRows,
  resolveTemplateVariableValues,
  storedTemplateDiagnosticMessages,
  stringifyTemplateMailMergePreviewRows,
  templateContentDiagnostics,
  templateDiagnosticsErrorMessage,
  templateFormSchema,
  templateMergeErrorMessage,
  templateFormToInput
} from './templateForm'

describe('template form', () => {
  it('extracts unique template variables in first-seen order', () => {
    expect(extractTemplateVariables(
      'Hello {{ recipient }}',
      '<p>{{body}}</p><p>{{ recipient }}</p><p>{{project.name}}</p>'
    )).toEqual(['recipient', 'body', 'project.name'])
  })

  it('normalizes compose content into a rich template save payload', () => {
    const values = templateFormSchema.parse({ name: '  Investor follow-up  ' })

    expect(templateFormToInput(values, {
      subject: 'Hello {{ recipient }}',
      body: 'Fallback {{ignored}}',
      bodyHtml: '<p>Next step for {{ project }}</p>'
    })).toEqual({
      name: 'Investor follow-up',
      subject_template: 'Hello {{ recipient }}',
      body_template: '<p>Next step for {{ project }}</p>',
      variables: ['recipient', 'project'],
      language: null
    })
  })

  it('preserves an existing template id when updating a template', () => {
    const values = templateFormSchema.parse({ name: 'Intro' })

    expect(templateFormToInput(values, {
      templateId: 'tpl-1',
      subject: 'Updated {{ name }}',
      body: 'Updated body',
      bodyHtml: null
    })).toMatchObject({
      template_id: 'tpl-1',
      name: 'Intro',
      variables: ['name']
    })
  })

  it('rejects empty template names before save', () => {
    expect(() => templateFormSchema.parse({ name: ' ' })).toThrow()
  })

  it('reports missing merge variables with stable copy', () => {
    const missing = missingTemplateVariables(['recipient', 'project', 'date'], {
      recipient: 'Alex',
      project: ' ',
      date: 'Jun 15, 2026'
    })

    expect(missing).toEqual(['project'])
    expect(templateMergeErrorMessage(missing)).toBe('Fill template variables: project')
  })

  it('reports malformed save placeholders before creating a rich template', () => {
    const diagnostics = templateContentDiagnostics(
      'Hello {{ recipient',
      '<p>{{ }} {{ project }} {{ first name }}</p>'
    )

    expect(diagnostics.variables).toEqual(['project'])
    expect(diagnostics.malformedPlaceholders).toEqual(['{{ recipient', '{{ }}', '{{ first name }}'])
    expect(templateDiagnosticsErrorMessage(diagnostics)).toBe(
      'Fix malformed template placeholders: {{ recipient, {{ }}, {{ first name }}'
    )
  })

  it('builds stored template diagnostic messages from backend metadata', () => {
    const messages = storedTemplateDiagnosticMessages({
      malformed_placeholders: ['{{ }}'],
      undeclared_variables: ['project'],
      unused_variables: ['legacy']
    })

    expect(messages).toEqual([
      {
        kind: 'error',
        label: 'Fix malformed placeholders',
        values: ['{{ }}']
      },
      {
        kind: 'error',
        label: 'Declare missing variables',
        values: ['project']
      },
      {
        kind: 'warning',
        label: 'Unused variables',
        values: ['legacy']
      }
    ])
  })

  it('derives stable default values for common template variables', () => {
    expect(defaultTemplateVariableValue('recipient', {
      toText: 'alex@example.com',
      ccText: 'team@example.com',
      bccText: 'audit@example.com',
      subject: 'Quarterly review',
      body: 'Body copy'
    })).toBe('alex@example.com')

    expect(defaultTemplateVariableValue('subject', {
      toText: '',
      ccText: '',
      bccText: '',
      subject: 'Quarterly review',
      body: 'Body copy'
    })).toBe('Quarterly review')
  })

  it('preserves existing variable values for the same template when requested', () => {
    expect(resolveTemplateVariableValues({
      variables: ['recipient', 'project', 'body']
    }, {
      recipient: 'alex@example.com',
      project: 'Макошь',
      body: ''
    }, {
      toText: 'default@example.com',
      ccText: '',
      bccText: '',
      subject: 'Quarterly review',
      body: 'Default body'
    }, {
      preserveExisting: true
    })).toEqual({
      recipient: 'alex@example.com',
      project: 'Макошь',
      body: 'Default body'
    })
  })

  it('parses JSON mail-merge preview rows with stable row ids', () => {
    expect(parseTemplateMailMergePreviewRows(`[
      { "row_id": "row-a", "variables": { "recipient": "alex@example.com", "project": "Макошь" } },
      { "recipient": "sam@example.com", "count": 2, "active": true }
    ]`)).toEqual([
      {
        row_id: 'row-a',
        variables: { recipient: 'alex@example.com', project: 'Макошь' }
      },
      {
        row_id: 'row-2',
        variables: { recipient: 'sam@example.com', count: '2', active: 'true' }
      }
    ])
  })

  it('stringifies preview rows and rejects invalid preview payloads', () => {
    expect(stringifyTemplateMailMergePreviewRows([
      { row_id: 'row-a', variables: { recipient: 'alex@example.com' } }
    ])).toContain('"row_id": "row-a"')

    expect(() => parseTemplateMailMergePreviewRows('{ "recipient": "alex@example.com" }')).toThrow(
      'Mail merge preview expects a JSON array of row objects'
    )
  })
})
```

### `frontend/src/domains/communications/forms/templateForm.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/forms/templateForm.ts`
- Size bytes / Размер в байтах: `8264`
- Included characters / Включено символов: `8264`
- Truncated / Обрезано: `no`

```typescript
import { toTypedSchema } from '@vee-validate/zod'
import { z } from 'zod'
import type { CommunicationTemplate, RichTemplateUpsertRequest } from '../types/templates'

export type TemplateFormValues = z.infer<typeof templateFormSchema>
export type TemplateComposeContent = {
  templateId?: string
  subject: string
  body: string
  bodyHtml: string | null
}
export type TemplateContentDiagnostics = {
  variables: string[]
  malformedPlaceholders: string[]
}
export type StoredTemplateDiagnosticMessage = {
  kind: 'error' | 'warning'
  label: string
  values: string[]
}
export type StoredTemplateDiagnosticSource = Pick<
  CommunicationTemplate,
  'malformed_placeholders' | 'undeclared_variables' | 'unused_variables'
>
export type TemplateVariableDefaultsContext = {
  toText: string
  ccText: string
  bccText: string
  subject: string
  body: string
}
export type TemplateMailMergePreviewRowInput = {
  row_id: string
  variables: Record<string, string>
}

const templateVariableNamePattern = /^[A-Za-z0-9_.-]+$/

export const templateFormSchema = z.object({
  name: z.string().trim().min(1, 'Template name is required').max(120, 'Template name is too long')
})

export const templateVeeValidationSchema = toTypedSchema(templateFormSchema)

export function templateFormDefaults(): TemplateFormValues {
  return {
    name: ''
  }
}

export function extractTemplateVariables(...sources: Array<string | null | undefined>): string[] {
  return templateContentDiagnostics(...sources).variables
}

export function templateContentDiagnostics(
  ...sources: Array<string | null | undefined>
): TemplateContentDiagnostics {
  const variables: string[] = []
  const malformedPlaceholders: string[] = []
  const seenVariables = new Set<string>()
  const seenMalformed = new Set<string>()

  for (const source of sources) {
    if (!source) continue
    inspectTemplateSource(source, variables, seenVariables, malformedPlaceholders, seenMalformed)
  }

  return { variables, malformedPlaceholders }
}

function inspectTemplateSource(
  source: string,
  variables: string[],
  seenVariables: Set<string>,
  malformedPlaceholders: string[],
  seenMalformed: Set<string>
): void {
  let rest = source
  while (true) {
    const start = rest.indexOf('{{')
    if (start === -1) return

    const afterOpen = rest.slice(start + 2)
    const end = afterOpen.indexOf('}}')
    if (end === -1) {
      addUnique(malformedPlaceholders, seenMalformed, rest.slice(start))
      return
    }

    const rawPlaceholder = rest.slice(start, start + 2 + end + 2)
    const variable = afterOpen.slice(0, end).trim()
    if (!variable || !templateVariableNamePattern.test(variable)) {
      addUnique(malformedPlaceholders, seenMalformed, rawPlaceholder)
    } else {
      addUnique(variables, seenVariables, variable)
    }
    rest = afterOpen.slice(end + 2)
  }
}

function addUnique(target: string[], seen: Set<string>, value: string): void {
  if (seen.has(value)) return
  seen.add(value)
  target.push(value)
}

export function missingTemplateVariables(
  variables: string[],
  values: Record<string, string>
): string[] {
  return variables.filter((variable) => !(values[variable] ?? '').trim())
}

export function templateMergeErrorMessage(missingVariables: string[]): string {
  if (!missingVariables.length) return ''
  return `Fill template variables: ${missingVariables.join(', ')}`
}

export function templateDiagnosticsErrorMessage(diagnostics: TemplateContentDiagnostics): string {
  if (!diagnostics.malformedPlaceholders.length) return ''
  return `Fix malformed template placeholders: ${diagnostics.malformedPlaceholders.join(', ')}`
}

export function storedTemplateDiagnosticMessages(
  template: StoredTemplateDiagnosticSource | null | undefined
): StoredTemplateDiagnosticMessage[] {
  if (!template) return []

  const messages: StoredTemplateDiagnosticMessage[] = []
  if (template.malformed_placeholders.length) {
    messages.push({
      kind: 'error',
      label: 'Fix malformed placeholders',
      values: template.malformed_placeholders
    })
  }
  if (template.undeclared_variables.length) {
    messages.push({
      kind: 'error',
      label: 'Declare missing variables',
      values: template.undeclared_variables
    })
  }
  if (template.unused_variables.length) {
    messages.push({
      kind: 'warning',
      label: 'Unused variables',
      values: template.unused_variables
    })
  }
  return messages
}

export function defaultTemplateVariableValue(
  variable: string,
  context: TemplateVariableDefaultsContext
): string {
  const normalized = variable.trim().toLowerCase()
  if (normalized === 'to' || normalized === 'recipient') return context.toText
  if (normalized === 'cc') return context.ccText
  if (normalized === 'bcc') return context.bccText
  if (normalized === 'subject') return context.subject
  if (normalized === 'body' || normalized === 'message') return context.body
  if (normalized === 'date' || normalized === 'current_date') {
    return new Intl.DateTimeFormat('en-US', { dateStyle: 'medium' }).format(new Date())
  }
  return ''
}

export function resolveTemplateVariableValues(
  template: Pick<CommunicationTemplate, 'variables'> | null | undefined,
  existingValues: Record<string, string>,
  context: TemplateVariableDefaultsContext,
  options: { preserveExisting: boolean }
): Record<string, string> {
  if (!template) return {}

  const resolved: Record<string, string> = {}
  for (const variable of template.variables) {
    const existingValue = existingValues[variable] ?? ''
    resolved[variable] = options.preserveExisting && existingValue.trim()
      ? existingValue
      : defaultTemplateVariableValue(variable, context)
  }
  return resolved
}

export function parseTemplateMailMergePreviewRows(
  rawValue: string
): TemplateMailMergePreviewRowInput[] {
  const trimmed = rawValue.trim()
  if (!trimmed) return []

  const parsed = JSON.parse(trimmed)
  if (!Array.isArray(parsed)) {
    throw new Error('Mail merge preview expects a JSON array of row objects')
  }

  return parsed.map((item, index) => normalizeTemplateMailMergePreviewRow(item, index))
}

export function stringifyTemplateMailMergePreviewRows(
  rows: TemplateMailMergePreviewRowInput[]
): string {
  return JSON.stringify(rows, null, 2)
}

export function templateFormToInput(
  values: TemplateFormValues,
  content: TemplateComposeContent
): RichTemplateUpsertRequest {
  const parsed = templateFormSchema.parse(values)
  const bodyTemplate = content.bodyHtml ?? content.body
  const diagnostics = templateContentDiagnostics(content.subject, bodyTemplate)
  return {
    ...(content.templateId?.trim() ? { template_id: content.templateId.trim() } : {}),
    name: parsed.name,
    subject_template: content.subject,
    body_template: bodyTemplate,
    variables: diagnostics.variables,
    language: null
  }
}

function normalizeTemplateMailMergePreviewRow(
  value: unknown,
  index: number
): TemplateMailMergePreviewRowInput {
  if (!value || typeof value !== 'object' || Array.isArray(value)) {
    throw new Error(`Mail merge preview row ${index + 1} must be an object`)
  }

  const record = value as Record<string, unknown>
  const rowId = typeof record.row_id === 'string' && record.row_id.trim()
    ? record.row_id.trim()
    : `row-${index + 1}`

  const variablesSource = 'variables' in record ? record.variables : record
  if (!variablesSource || typeof variablesSource !== 'object' || Array.isArray(variablesSource)) {
    throw new Error(`Mail merge preview row ${index + 1} must provide an object of variables`)
  }

  const variables: Record<string, string> = {}
  for (const [key, rawVariableValue] of Object.entries(variablesSource as Record<string, unknown>)) {
    if (key === 'row_id') continue
    if (rawVariableValue === null || rawVariableValue === undefined) {
      variables[key] = ''
      continue
    }
    if (typeof rawVariableValue === 'string' || typeof rawVariableValue === 'number' || typeof rawVariableValue === 'boolean') {
      variables[key] = String(rawVariableValue)
      continue
    }
    throw new Error(`Mail merge preview row ${index + 1} variable "${key}" must be a string, number, boolean, null, or omitted`)
  }

  return {
    row_id: rowId,
    variables
  }
}
```

### `frontend/src/domains/communications/helpers/communicationPageModels.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/helpers/communicationPageModels.test.ts`
- Size bytes / Размер в байтах: `11406`
- Included characters / Включено символов: `11406`
- Truncated / Обрезано: `no`

```typescript
import { describe, expect, it } from 'vitest'
import type { CommunicationMessageSummary, CommunicationDraft, ThreadMessage } from '../types/communications'
import {
  aiSummaryContractFromMetadata,
  composeFormToSendRequest,
  draftToComposeForm,
  emptyCommunicationMessageInsight,
  communicationMessageLabelsFromMetadata,
  communicationMessageSnoozeUntilFromMetadata,
  communicationKnowledgeSectionsFromSummaryContract,
  communicationExtractionSectionsFromInsight,
  forwardComposeForm,
  newComposeForm,
  replyComposeForm,
  replyAllComposeForm,
  threadReplyComposeForm
} from './communicationPageModels'

function message(overrides: Partial<CommunicationMessageSummary> = {}): CommunicationMessageSummary {
  return {
    message_id: 'msg-1',
    raw_record_id: 'raw-1',
    account_id: 'account-1',
    provider_record_id: 'provider-1',
    subject: 'Quarterly update',
    sender: 'alice@example.com',
    recipients: ['owner@example.com'],
    body_text_preview: 'Preview',
    occurred_at: null,
    projected_at: '2026-06-15T10:00:00Z',
    channel_kind: 'email',
    conversation_id: null,
    sender_display_name: null,
    delivery_state: 'received',
    workflow_state: 'new',
    importance_score: null,
    ai_category: null,
    ai_summary: null,
    ai_summary_generated_at: null,
    message_metadata: {},
    attachment_count: 0,
    local_state: 'active',
    local_state_changed_at: null,
    ...overrides
  }
}

function draft(): CommunicationDraft {
  return {
    draft_id: 'draft-1',
    account_id: 'account-1',
    persona_id: null,
    to_recipients: ['to@example.com'],
    cc_recipients: ['cc@example.com'],
    bcc_recipients: ['bcc@example.com'],
    subject: 'Draft subject',
    body_text: 'Draft body',
    body_html: null,
    in_reply_to: 'provider-1',
    references: [],
    status: 'draft',
    scheduled_send_at: null,
    send_attempts: 0,
    last_error: null,
    metadata: {},
    created_at: '2026-06-15T10:00:00Z',
    updated_at: '2026-06-15T10:01:00Z'
  }
}

function threadMessage(overrides: Partial<ThreadMessage> = {}): ThreadMessage {
  return {
    message_id: 'thread-msg-1',
    provider_record_id: 'provider-thread-1',
    account_id: 'account-1',
    subject: 'Quarterly update',
    sender: 'Ada <ada@example.com>',
    sender_display_name: 'Ada',
    body_text: 'Line one\nLine two with <angle>',
    occurred_at: null,
    projected_at: '2026-06-15T10:00:00Z',
    workflow_state: 'new',
    importance_score: null,
    ai_category: null,
    ai_summary: null,
    delivery_state: 'received',
    attachment_count: 0,
    attachments: [],
    ...overrides
  }
}

describe('mail page model helpers', () => {
  it('creates an empty message insight shell for the selected message', () => {
    expect(emptyCommunicationMessageInsight('msg-1')).toMatchObject({
      messageId: 'msg-1',
      tasks: [],
      notes: [],
      translation: null
    })
  })

  it('extracts structured AI summary contracts from message metadata safely', () => {
    expect(aiSummaryContractFromMetadata({
      ai_summary_contract: {
        key_points: ['Contract review'],
        action_items: ['Reply by Friday'],
        risks: ['Payment risk'],
        deadlines: ['Friday'],
        event_candidates: [{ title: 'Review meeting', evidence: 'Meeting on Monday' }],
        persona_candidates: [{ title: 'Ada Lovelace', evidence: 'ada@example.com' }],
        organization_candidates: [{ title: 'Acme Corp', evidence: 'acme.example' }],
        document_candidates: [{ title: 'MSA attachment', evidence: 'attached MSA' }],
        agreement_candidates: [{ title: 'NDA', evidence: 'review NDA' }]
      }
    })).toEqual({
      key_points: ['Contract review'],
      action_items: ['Reply by Friday'],
      risks: ['Payment risk'],
      deadlines: ['Friday'],
      event_candidates: [{ title: 'Review meeting', evidence: 'Meeting on Monday' }],
      persona_candidates: [{ title: 'Ada Lovelace', evidence: 'ada@example.com' }],
      organization_candidates: [{ title: 'Acme Corp', evidence: 'acme.example' }],
      document_candidates: [{ title: 'MSA attachment', evidence: 'attached MSA' }],
      agreement_candidates: [{ title: 'NDA', evidence: 'review NDA' }]
    })

    expect(aiSummaryContractFromMetadata({
      ai_summary_contract: {
        key_points: ['ok', 42],
        action_items: 'not-array',
        risks: [],
        deadlines: [null],
        event_candidates: ['legacy event candidate', { title: 42 }],
        persona_candidates: [{ title: 'Ada', evidence: 42 }],
        organization_candidates: null,
        document_candidates: [{ title: 'Doc' }],
        agreement_candidates: [{ evidence: 'Missing title' }]
      }
    })).toEqual({
      key_points: ['ok'],
      action_items: [],
      risks: [],
      deadlines: [],
      event_candidates: [{ title: 'legacy event candidate', evidence: 'legacy event candidate' }],
      persona_candidates: [{ title: 'Ada', evidence: '' }],
      organization_candidates: [],
      document_candidates: [{ title: 'Doc', evidence: '' }],
      agreement_candidates: []
    })

    expect(aiSummaryContractFromMetadata({})).toBeNull()
  })

  it('builds mail knowledge review sections from AI summary candidates', () => {
    const contract = aiSummaryContractFromMetadata({
      ai_summary_contract: {
        key_points: [],
        action_items: [],
        risks: [],
        deadlines: [],
        event_candidates: [{ title: 'Review meeting', evidence: 'Monday 10:00' }],
        persona_candidates: [{ title: 'Ada Lovelace', evidence: 'ada@example.com' }],
        organization_candidates: [{ title: 'Acme Corp', evidence: 'acme.example' }],
        document_candidates: [{ title: 'MSA attachment', evidence: 'attached MSA' }],
        agreement_candidates: [{ title: 'NDA', evidence: 'review NDA' }]
      }
    })

    expect(communicationKnowledgeSectionsFromSummaryContract(contract)).toEqual([
      {
        kind: 'event',
        title: 'Event candidates',
        items: [{ title: 'Review meeting', evidence: 'Monday 10:00' }]
      },
      {
        kind: 'persona',
        title: 'Persona candidates',
        items: [{ title: 'Ada Lovelace', evidence: 'ada@example.com' }]
      },
      {
        kind: 'organization',
        title: 'Organization candidates',
        items: [{ title: 'Acme Corp', evidence: 'acme.example' }]
      },
      {
        kind: 'document',
        title: 'Document candidates',
        items: [{ title: 'MSA attachment', evidence: 'attached MSA' }]
      },
      {
        kind: 'agreement',
        title: 'Agreement candidates',
        items: [{ title: 'NDA', evidence: 'review NDA' }]
      }
    ])
  })

  it('extracts message labels and snooze metadata safely', () => {
    expect(communicationMessageLabelsFromMetadata({
      labels: ['finance', ' urgent ', 42, '', 'finance']
    })).toEqual(['finance', 'urgent'])

    expect(communicationMessageLabelsFromMetadata({ labels: 'finance' })).toEqual([])
    expect(communicationMessageSnoozeUntilFromMetadata({ snooze_until: '2026-06-20T10:00:00Z' }))
      .toBe('2026-06-20T10:00:00Z')
    expect(communicationMessageSnoozeUntilFromMetadata({ snooze_until: 42 })).toBeNull()
  })

  it('builds review sections for extracted mail task and note candidates', () => {
    const sections = communicationExtractionSectionsFromInsight({
      ...emptyCommunicationMessageInsight('msg-1'),
      tasks: [
        {
          title: 'Send signed amendment',
          due_date: '2026-06-20',
          assignee: 'Ada',
          priority: 'high',
          source: 'Please send the signed amendment by Friday.'
        }
      ],
      notes: [
        {
          title: 'Commercial terms',
          content: 'Discount applies after renewal.',
          tags: ['contract', 'renewal'],
          source: 'Renewal clause'
        }
      ]
    })

    expect(sections).toEqual([
      {
        kind: 'task',
        title: 'Task candidates',
        items: [
          {
            title: 'Send signed amendment',
            meta: ['Due 2026-06-20', 'Assignee Ada', 'Priority high'],
            body: 'Please send the signed amendment by Friday.'
          }
        ]
      },
      {
        kind: 'note',
        title: 'Note candidates',
        items: [
          {
            title: 'Commercial terms',
            meta: ['contract', 'renewal'],
            body: 'Discount applies after renewal.'
          }
        ]
      }
    ])

    expect(communicationExtractionSectionsFromInsight(null)).toEqual([])
  })

  it('builds compose form models for new, reply and persisted draft flows', () => {
    expect(newComposeForm('account-1', 'draft-new')).toMatchObject({
      mode: 'compose',
      draftId: 'draft-new',
      accountId: 'account-1',
      subject: ''
    })
    expect(replyComposeForm(message(), 'fallback-account', 'draft-reply')).toMatchObject({
      mode: 'reply',
      draftId: 'draft-reply',
      accountId: 'account-1',
      toText: 'alice@example.com',
      subject: 'Re: Quarterly update',
      inReplyTo: 'provider-1'
    })
    expect(replyAllComposeForm(message({ recipients: ['owner@example.com', 'team@example.com'] }), 'fallback-account', 'draft-reply-all')).toMatchObject({
      mode: 'reply',
      draftId: 'draft-reply-all',
      toText: 'alice@example.com',
      ccText: 'owner@example.com, team@example.com',
      subject: 'Re: Quarterly update',
      inReplyTo: 'provider-1'
    })
    expect(forwardComposeForm(message(), 'fallback-account', 'draft-forward')).toMatchObject({
      mode: 'forward',
      draftId: 'draft-forward',
      toText: '',
      subject: 'Fwd: Quarterly update'
    })
    expect(draftToComposeForm(draft())).toMatchObject({
      draftId: 'draft-1',
      toText: 'to@example.com',
      ccText: 'cc@example.com',
      bccText: 'bcc@example.com',
      body: 'Draft body'
    })
  })

  it('builds quoted rich compose models for thread message replies', () => {
    const form = threadReplyComposeForm(
      threadMessage(),
      'fallback-account',
      'draft-thread-reply',
      '<p>Inline draft</p>'
    )

    expect(form).toMatchObject({
      mode: 'reply',
      draftId: 'draft-thread-reply',
      accountId: 'account-1',
      toText: 'Ada <ada@example.com>',
      subject: 'Re: Quarterly update',
      bodyFormat: 'html',
      inReplyTo: 'provider-thread-1'
    })
    expect(form.body).toContain('On 2026-06-15T10:00:00Z, Ada <ada@example.com> wrote:')
    expect(form.body).toContain('Inline draft')
    expect(form.body).toContain('> Line one')
    expect(form.bodyHtml ?? '').toContain('<p>Inline draft</p>')
    expect(form.bodyHtml ?? '').toContain('<blockquote')
    expect(form.bodyHtml ?? '').toContain('Line one<br>Line two with &lt;angle&gt;')
  })

  it('converts compose models into provider-write send requests', () => {
    const form = threadReplyComposeForm(
      threadMessage(),
      'fallback-account',
      'draft-thread-reply',
      '<p>Inline draft</p>'
    )
    const request = composeFormToSendRequest(form)

    expect(request).toMatchObject({
      account_id: 'account-1',
      to: ['ada@example.com'],
      subject: 'Re: Quarterly update',
      draft_id: 'draft-thread-reply',
      in_reply_to: 'provider-thread-1',
      confirmed_provider_write: true
    })
    expect(request.body_html).toContain('<p>Inline draft</p>')
  })
})
```

### `frontend/src/domains/communications/helpers/communicationPageModels.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/helpers/communicationPageModels.ts`
- Size bytes / Размер в байтах: `11052`
- Included characters / Включено символов: `11052`
- Truncated / Обрезано: `no`

```typescript
import type {
  CommunicationMessageSummary,
  ComposeFormModel,
  CommunicationDraft,
  CommunicationKnowledgeCandidate,
  CommunicationMessageInsight,
  SendCommunicationRequest,
  ThreadMessage
} from '../types/communications'
import { datetimeLocalToIso } from '../forms/composeDraftAutosave'
import { splitComposeRecipients } from '../forms/composeValidation'

export type AiSummaryContract = {
  key_points: string[]
  action_items: string[]
  risks: string[]
  deadlines: string[]
  event_candidates: CommunicationKnowledgeCandidate[]
  persona_candidates: CommunicationKnowledgeCandidate[]
  organization_candidates: CommunicationKnowledgeCandidate[]
  document_candidates: CommunicationKnowledgeCandidate[]
  agreement_candidates: CommunicationKnowledgeCandidate[]
}

export type CommunicationExtractionReviewItem = {
  title: string
  meta: string[]
  body: string
}

export type CommunicationExtractionReviewSection = {
  kind: 'task' | 'note'
  title: string
  items: CommunicationExtractionReviewItem[]
}

export type CommunicationKnowledgeReviewSection = {
  kind: 'event' | 'persona' | 'organization' | 'document' | 'agreement'
  title: string
  items: CommunicationKnowledgeCandidate[]
}

export function emptyCommunicationMessageInsight(messageId: string): CommunicationMessageInsight {
  return {
    messageId,
    explain: null,
    smartCc: null,
    auth: null,
    signature: null,
    language: null,
    aiReply: null,
    tasks: [],
    notes: [],
    translation: null
  }
}

export function communicationExtractionSectionsFromInsight(
  insight: CommunicationMessageInsight | null
): CommunicationExtractionReviewSection[] {
  if (!insight) return []
  const sections: CommunicationExtractionReviewSection[] = []
  if (insight.tasks.length > 0) {
    sections.push({
      kind: 'task',
      title: 'Task candidates',
      items: insight.tasks.map((task) => ({
        title: task.title,
        meta: [
          task.due_date ? `Due ${task.due_date}` : '',
          task.assignee ? `Assignee ${task.assignee}` : '',
          task.priority ? `Priority ${task.priority}` : ''
        ].filter(nonEmptyString),
        body: task.source
      }))
    })
  }
  if (insight.notes.length > 0) {
    sections.push({
      kind: 'note',
      title: 'Note candidates',
      items: insight.notes.map((note) => ({
        title: note.title,
        meta: note.tags.filter(nonEmptyString),
        body: note.content.trim() || note.source
      }))
    })
  }
  return sections
}

export function communicationKnowledgeSectionsFromSummaryContract(
  contract: AiSummaryContract | null
): CommunicationKnowledgeReviewSection[] {
  if (!contract) return []
  return [
    { kind: 'event' as const, title: 'Event candidates', items: contract.event_candidates },
    { kind: 'persona' as const, title: 'Persona candidates', items: contract.persona_candidates },
    {
      kind: 'organization' as const,
      title: 'Organization candidates',
      items: contract.organization_candidates
    },
    { kind: 'document' as const, title: 'Document candidates', items: contract.document_candidates },
    { kind: 'agreement' as const, title: 'Agreement candidates', items: contract.agreement_candidates }
  ].filter((section) => section.items.length > 0)
}

export function communicationMessageLabelsFromMetadata(metadata: Record<string, unknown>): string[] {
  const labels = metadata.labels
  if (!Array.isArray(labels)) return []
  return [...new Set(labels
    .filter((label): label is string => typeof label === 'string' && label.trim().length > 0)
    .map((label) => label.trim()))]
}

export function communicationMessageSnoozeUntilFromMetadata(metadata: Record<string, unknown>): string | null {
  return typeof metadata.snooze_until === 'string' && metadata.snooze_until.trim().length > 0
    ? metadata.snooze_until.trim()
    : null
}

export function aiSummaryContractFromMetadata(metadata: Record<string, unknown>): AiSummaryContract | null {
  const value = metadata.ai_summary_contract
  if (!isRecord(value)) return null
  return {
    key_points: stringArrayValue(value.key_points),
    action_items: stringArrayValue(value.action_items),
    risks: stringArrayValue(value.risks),
    deadlines: stringArrayValue(value.deadlines),
    event_candidates: candidateArrayValue(value.event_candidates),
    persona_candidates: candidateArrayValue(value.persona_candidates),
    organization_candidates: candidateArrayValue(value.organization_candidates),
    document_candidates: candidateArrayValue(value.document_candidates),
    agreement_candidates: candidateArrayValue(value.agreement_candidates)
  }
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null && !Array.isArray(value)
}

function stringArrayValue(value: unknown): string[] {
  if (!Array.isArray(value)) return []
  return value.filter((item): item is string => typeof item === 'string' && item.trim().length > 0)
}

function candidateArrayValue(value: unknown): CommunicationKnowledgeCandidate[] {
  if (!Array.isArray(value)) return []
  return value.flatMap((item): CommunicationKnowledgeCandidate[] => {
    if (typeof item === 'string' && item.trim().length > 0) {
      const candidate = item.trim()
      return [{ title: candidate, evidence: candidate }]
    }
    if (!isRecord(item) || typeof item.title !== 'string' || item.title.trim().length === 0) {
      return []
    }
    return [{
      title: item.title.trim(),
      evidence: typeof item.evidence === 'string' ? item.evidence.trim() : ''
    }]
  })
}

function nonEmptyString(value: string): value is string {
  return value.trim().length > 0
}

export function replyComposeForm(
  message: CommunicationMessageSummary,
  fallbackAccountId: string,
  draftId: string
): ComposeFormModel {
  return {
    mode: 'reply',
    draftId,
    accountId: message.account_id || fallbackAccountId,
    toText: message.sender,
    ccText: '',
    bccText: '',
    subject: message.subject.startsWith('Re:') ? message.subject : `Re: ${message.subject}`,
    body: '',
    bodyHtml: null,
    bodyFormat: 'plain',
    scheduledSendAt: '',
    undoSendSeconds: null,
    inReplyTo: message.provider_record_id || null
  }
}

export function replyAllComposeForm(
  message: CommunicationMessageSummary,
  fallbackAccountId: string,
  draftId: string
): ComposeFormModel {
  return {
    ...replyComposeForm(message, fallbackAccountId, draftId),
    ccText: message.recipients.join(', ')
  }
}

export function forwardComposeForm(
  message: CommunicationMessageSummary,
  fallbackAccountId: string,
  draftId: string
): ComposeFormModel {
  const subject = message.subject.startsWith('Fwd:') ? message.subject : `Fwd: ${message.subject}`
  const body = [
    '',
    '',
    '--- Forwarded message ---',
    `From: ${message.sender}`,
    `Subject: ${message.subject}`,
    '',
    message.body_text_preview
  ].join('\n')
  return {
    mode: 'forward',
    draftId,
    accountId: message.account_id || fallbackAccountId,
    toText: '',
    ccText: '',
    bccText: '',
    subject,
    body,
    bodyHtml: null,
    bodyFormat: 'plain',
    scheduledSendAt: '',
    undoSendSeconds: null,
    inReplyTo: null
  }
}

export function threadReplyComposeForm(
  message: ThreadMessage,
  fallbackAccountId: string,
  draftId: string,
  draftBodyHtml = ''
): ComposeFormModel {
  const quotedText = quotedPlainText(message)
  const normalizedDraftHtml = draftBodyHtml.trim()
  return {
    mode: 'reply',
    draftId,
    accountId: message.account_id || fallbackAccountId,
    toText: message.sender,
    ccText: '',
    bccText: '',
    subject: message.subject.startsWith('Re:') ? message.subject : `Re: ${message.subject}`,
    body: normalizedDraftHtml
      ? `${htmlToPlainText(normalizedDraftHtml)}${quotedText}`
      : quotedText,
    bodyHtml: normalizedDraftHtml
      ? `${normalizedDraftHtml}${quotedHtml(message)}`
      : quotedHtml(message),
    bodyFormat: 'html',
    scheduledSendAt: '',
    undoSendSeconds: null,
    inReplyTo: message.provider_record_id || message.message_id
  }
}

export function newComposeForm(accountId: string, draftId: string): ComposeFormModel {
  return {
    mode: 'compose',
    draftId,
    accountId,
    toText: '',
    ccText: '',
    bccText: '',
    subject: '',
    body: '',
    bodyHtml: null,
    bodyFormat: 'plain',
    scheduledSendAt: '',
    undoSendSeconds: null,
    inReplyTo: null
  }
}

export function composeFormToSendRequest(form: ComposeFormModel): SendCommunicationRequest {
  return {
    account_id: form.accountId,
    to: splitComposeRecipients(form.toText),
    cc: splitComposeRecipients(form.ccText),
    bcc: splitComposeRecipients(form.bccText),
    subject: form.subject,
    body_text: form.body,
    body_html: form.bodyFormat === 'html' ? form.bodyHtml : null,
    in_reply_to: form.inReplyTo,
    draft_id: form.draftId,
    scheduled_send_at: datetimeLocalToIso(form.scheduledSendAt),
    undo_send_seconds: form.undoSendSeconds,
    confirmed_provider_write: true
  }
}

function quotedPlainText(message: ThreadMessage): string {
  const header = `On ${message.projected_at}, ${message.sender} wrote:`
  const quoted = message.body_text
    .split(/\r?\n/)
    .map((line) => `> ${line}`)
    .join('\n')
  return `\n\n${header}\n${quoted}`
}

function quotedHtml(message: ThreadMessage): string {
  const body = escapeHtml(message.body_text).replace(/\r?\n/g, '<br>')
  return [
    '<p><br></p>',
    `<p>On ${escapeHtml(message.projected_at)}, ${escapeHtml(message.sender)} wrote:</p>`,
    `<blockquote>${body}</blockquote>`
  ].join('')
}

function escapeHtml(value: string): string {
  return value
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#39;')
}

function htmlToPlainText(value: string): string {
  return value
    .replace(/<br\s*\/?>/gi, '\n')
    .replace(/<\/p>/gi, '\n')
    .replace(/<[^>]+>/g, '')
    .replace(/&lt;/g, '<')
    .replace(/&gt;/g, '>')
    .replace(/&quot;/g, '"')
    .replace(/&#39;/g, "'")
    .replace(/&amp;/g, '&')
    .trim()
}

export function draftToComposeForm(draft: CommunicationDraft): ComposeFormModel {
  return {
    mode: 'compose',
    draftId: draft.draft_id,
    accountId: draft.account_id,
    toText: draft.to_recipients.join(', '),
    ccText: draft.cc_recipients.join(', '),
    bccText: draft.bcc_recipients.join(', '),
    subject: draft.subject,
    body: draft.body_text,
    bodyHtml: draft.body_html,
    bodyFormat: draft.body_html ? 'html' : 'plain',
    scheduledSendAt: isoToDatetimeLocal(draft.scheduled_send_at),
    undoSendSeconds: null,
    inReplyTo: draft.in_reply_to
  }
}

function isoToDatetimeLocal(value: string | null): string {
  if (!value) return ''
  const date = new Date(value)
  if (!Number.isFinite(date.getTime())) return ''
  const offsetMs = date.getTimezoneOffset() * 60_000
  return new Date(date.getTime() - offsetMs).toISOString().slice(0, 16)
}
```

### `frontend/src/domains/communications/providers/whatsapp/views/WhatsAppCommunicationsPanel.boundary.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/providers/whatsapp/views/WhatsAppCommunicationsPanel.boundary.test.ts`
- Size bytes / Размер в байтах: `4133`
- Included characters / Включено символов: `4133`
- Truncated / Обрезано: `no`

```typescript
import { readFileSync } from 'node:fs'
import { describe, expect, it } from 'vitest'

function readSource(relativePath: string): string {
  return readFileSync(new URL(relativePath, import.meta.url), 'utf8')
}

describe('WhatsAppCommunicationsPanel boundary', () => {
  it('uses an in-panel forward target selector', () => {
    const source = readSource('./WhatsAppCommunicationsPanel.vue')
    const detailPaneSource = readSource('./WhatsAppCommunicationsDetailPane.vue')

    expect(source).not.toContain("t('Forward to conversation id')")
    expect(detailPaneSource).toContain("Forward target")
    expect(detailPaneSource).toContain("Filter target conversations")
    expect(detailPaneSource).toContain("Forward here")
  })

  it('uses an in-panel edit flow instead of a prompt', () => {
    const source = readSource('./WhatsAppCommunicationsPanel.vue')
    const detailPaneSource = readSource('./WhatsAppCommunicationsDetailPane.vue')

    expect(source).not.toContain("window.prompt(t('Edit message')")
    expect(detailPaneSource).toContain("Edit draft")
    expect(detailPaneSource).toContain("Edited text")
    expect(detailPaneSource).toContain("Save edit")
  })

  it('supports jumping from pinned and media sections back to timeline messages', () => {
    const source = readSource('./WhatsAppCommunicationsPanel.vue')
    const detailPaneSource = readSource('./WhatsAppCommunicationsDetailPane.vue')

    expect(source).toContain('jumpToMessage')
    expect(detailPaneSource).toContain("Jump to message")
    expect(detailPaneSource).toContain("Open source message")
  })

  it('exposes a dedicated media browsing mode with kind filtering', () => {
    const source = readFileSync(new URL('./WhatsAppCommunicationsPanel.vue', import.meta.url), 'utf8')

    expect(source).toContain("browserMode")
    expect(source).toContain("Timeline")
    expect(source).toContain("All media")
    expect(source).toContain("Images")
    expect(source).toContain("Videos")
    expect(source).toContain("Documents")
  })

  it('renders a safe in-panel media preview surface for projected attachments', () => {
    const source = readSource('./WhatsAppCommunicationsPanel.vue')
    const detailPaneSource = readSource('./WhatsAppCommunicationsDetailPane.vue')

    expect(source).toContain('useAttachmentPreviewQuery')
    expect(detailPaneSource).toContain("Media preview")
    expect(detailPaneSource).toContain("Preview media")
    expect(detailPaneSource).toContain("Select previewable media to open it here.")
    expect(detailPaneSource).toContain("mediaPreview.preview_kind === 'audio'")
    expect(detailPaneSource).toContain("mediaPreview.preview_kind === 'video'")
    expect(detailPaneSource).toContain("mediaPreview.preview_kind === 'pdf'")
  })

  it('renders projected rich WhatsApp message metadata', () => {
    const chatPaneSource = readSource('./WhatsAppCommunicationsChatPane.vue')
    const helpersSource = readSource('./WhatsAppCommunicationsPanel.helpers.ts')

    expect(helpersSource).toContain('whatsapp_link_preview')
    expect(helpersSource).toContain('whatsapp_poll')
    expect(helpersSource).toContain('whatsapp_location')
    expect(helpersSource).toContain('whatsapp_contact_card')
    expect(helpersSource).toContain('whatsapp_sticker')
    expect(helpersSource).toContain('whatsapp_view_once')
    expect(helpersSource).toContain('whatsapp_ephemeral')
    expect(chatPaneSource).toContain('messageLinkPreview')
    expect(chatPaneSource).toContain('messagePollSummary')
  })

  it('renders projected status lifecycle and status-media details in the timeline', () => {
    const chatPaneSource = readSource('./WhatsAppCommunicationsChatPane.vue')
    const helpersSource = readSource('./WhatsAppCommunicationsPanel.helpers.ts')

    expect(helpersSource).toContain('status_view_count')
    expect(helpersSource).toContain('status_last_viewer_display_name')
    expect(helpersSource).toContain('status_deleted_at')
    expect(helpersSource).toContain('status_author_business_profile')
    expect(chatPaneSource).toContain('Status author')
    expect(chatPaneSource).toContain('Status media')
  })
})
```

### `frontend/src/domains/communications/providers/whatsapp/views/WhatsAppCommunicationsPanel.helpers.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/providers/whatsapp/views/WhatsAppCommunicationsPanel.helpers.ts`
- Size bytes / Размер в байтах: `11561`
- Included characters / Включено символов: `11553`
- Truncated / Обрезано: `no`

```typescript
import type {
	WhatsappWebMediaItem,
} from '../../../../../shared/communications/types/whatsapp'
import type { TelegramReactionGroup } from '../../../../../shared/communications/types/telegram'
import type { TelegramChatMember } from '../../../../../shared/communications/types/telegramMembers'

type Translate = (key: string) => string

export type WhatsAppPanelMessage = {
	message_id: string
	raw_record_id?: string
	account_id: string
	provider_record_id?: string
	provider_message_id?: string
	provider_chat_id?: string | null
	conversation_id?: string | null
	chat_title?: string
	sender: string
	sender_display_name: string | null
	text?: string
	body_text_preview?: string | null
	occurred_at: string | null
	projected_at: string
	channel_kind?: string
	delivery_state: string
	metadata?: Record<string, unknown>
	message_metadata?: Record<string, unknown>
}

export function messageTime(message: WhatsAppPanelMessage): string {
	const value = message.occurred_at ?? message.projected_at
	if (!value) return ''
	const date = new Date(value)
	return Number.isNaN(date.getTime())
		? ''
		: new Intl.DateTimeFormat('en', { month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit' }).format(date)
}

export function memberLabel(member: TelegramChatMember): string {
	return member.sender_display_name ?? member.username ?? member.sender_id
}

export function mediaLabel(item: WhatsappWebMediaItem): string {
	return item.file_name || item.provider_attachment_id || item.kind
}

export function mediaAttachmentId(item: WhatsappWebMediaItem): string | null {
	return item.attachment_id?.trim() || null
}

export function isPreviewableMediaItem(item: WhatsappWebMediaItem): boolean {
	const attachmentId = mediaAttachmentId(item)
	if (!attachmentId) return false
	const mime = item.mime_type?.toLowerCase() ?? ''
	const fileName = item.file_name?.toLowerCase() ?? ''
	return (
		mime.startsWith('image/') ||
		mime.startsWith('audio/') ||
		mime.startsWith('video/') ||
		mime.startsWith('text/') ||
		mime === 'application/pdf' ||
		mime === 'application/json' ||
		mime === 'application/xml' ||
		mime === 'text/csv' ||
		fileName.endsWith('.pdf')
	)
}

export function firstPreviewableMediaAttachmentId(items: WhatsappWebMediaItem[]): string | null {
	return items.find((item) => isPreviewableMediaItem(item))?.attachment_id ?? null
}

export function mediaMetaLabel(item: WhatsappWebMediaItem): string {
	const parts = [item.kind]
	if (item.mime_type) parts.push(item.mime_type)
	if (item.download_state) parts.push(item.download_state)
	return parts.join(' · ')
}

export function mediaTime(item: WhatsappWebMediaItem): string {
	const value = item.occurred_at
	if (!value) return ''
	const date = new Date(value)
	return Number.isNaN(date.getTime())
		? ''
		: new Intl.DateTimeFormat('en', {
			month: 'short',
			day: 'numeric',
			hour: '2-digit',
			minute: '2-digit',
		}).format(date)
}

export function statusMessageMediaItems(
	message: WhatsAppPanelMessage,
	mediaItems: WhatsappWebMediaItem[]
): WhatsappWebMediaItem[] {
	return mediaItems.filter((item) => item.message_id === message.message_id)
}

export function statusAuthorHeadline(message: WhatsAppPanelMessage): string | null {
	const metadata = messageMetadata(message)
	return (
		metadataString(metadata.status_author_push_name) ||
		message.sender_display_name ||
		metadataString(metadata.status_author_address) ||
		message.sender
	)
}

export function statusAuthorDetail(message: WhatsAppPanelMessage): string | null {
	const metadata = messageMetadata(message)
	const parts: string[] = []
	const identityKind = metadataString(metadata.status_author_identity_kind)
	const address = metadataString(metadata.status_author_address)
	const businessProfile = metadataRecord(metadata.status_author_business_profile)
	const businessLabel = businessProfile
		? metadataString(
			businessProfile.verified_name ??
			businessProfile.business_name ??
			businessProfile.category ??
			businessProfile.description
		)
		: null
	if (identityKind) parts.push(identityKind)
	if (address) parts.push(address)
	if (businessLabel) parts.push(businessLabel)
	return parts.length ? parts.join(' · ') : null
}

export function statusViewSummary(message: WhatsAppPanelMessage, t: Translate): string | null {
	const metadata = messageMetadata(message)
	const count =
		typeof metadata.status_view_count === 'number'
			? metadata.status_view_count
			: null
	const lastViewer =
		metadataString(metadata.status_last_viewer_display_name) ||
		metadataString(metadata.status_last_viewer_id)
	if (count != null && lastViewer) return `${count} ${t('views')} · ${t('Last viewer')}: ${lastViewer}`
	if (count != null) return `${count} ${t('views')}`
	if (metadata.status_viewed) return t('Viewed')
	return null
}

export function statusDeletedSummary(message: WhatsAppPanelMessage, t: Translate): string | null {
	const metadata = messageMetadata(message)
	if (!metadata.status_deleted) return null
	const deletedAt = metadataString(metadata.status_deleted_at)
	if (!deletedAt) return t('Deleted')
	const date = new Date(deletedAt)
	return Number.isNaN(date.getTime())
		? `${t('Deleted')} · ${deletedAt}`
		: `${t('Deleted')} · ${new Intl.DateTimeFormat('en', { month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit' }).format(date)}`
}

export function statusMediaCountLabel(
	message: WhatsAppPanelMessage,
	mediaItems: WhatsappWebMediaItem[],
	t: Translate
): string | null {
	const count = statusMessageMediaItems(message, mediaItems).length
	if (!count) return null
	return count === 1 ? t('1 media item') : `${count} ${t('media items')}`
}

export function messagePreview(
	message: { text?: string; body_text_preview?: string | null },
	t: Translate
): string {
	return message.text || message.body_text_preview || t('No preview')
}

function metadataRecord(value: unknown): Record<string, unknown> | null {
	return typeof value === 'object' && value !== null && !Array.isArray(value)
		? (value as Record<string, unknown>)
		: null
}

function metadataString(value: unknown): string | null {
	return typeof value === 'string' && value.trim() ? value.trim() : null
}

function metadataArray(value: unknown): unknown[] {
	return Array.isArray(value) ? value : []
}

function messageMetadata(message: WhatsAppPanelMessage): Record<string, unknown> {
	return message.metadata ?? message.message_metadata ?? {}
}

export function messageMetaFlags(message: WhatsAppPanelMessage, t: Translate): string[] {
	const metadata = messageMetadata(message)
	const flags: string[] = []
	if (typeof metadata.mention_count === 'number' && metadata.mention_count > 0) {
		flags.push(`@${metadata.mention_count}`)
	}
	if (metadata.whatsapp_view_once) flags.push(t('View once'))
	if (metadata.whatsapp_ephemeral) flags.push(t('Ephemeral'))
	if (metadata.whatsapp_sticker) flags.push(t('Sticker'))
	if (metadata.whatsapp_poll) flags.push(t('Poll'))
	if (metadata.whatsapp_location) flags.push(t('Location'))
	if (metadata.whatsapp_contact_card) flags.push(t('Contact card'))
	if (metadata.whatsapp_system_message) flags.push(t('System'))
	if (metadata.whatsapp_join_leave) flags.push(t('Membership'))
	if (metadata.whatsapp_link_preview) flags.push(t('Link'))
	if (metadata.communication_object_type === 'status') flags.push(t('Status'))
	return flags
}

export function isStatusMessage(message: WhatsAppPanelMessage): boolean {
	const metadata = messageMetadata(message)
	return (
		metadata.communication_object_type === 'status' ||
		message.provider_chat_id === 'status-feed'
	)
}

export function messageMentionNames(message: WhatsAppPanelMessage): string[] {
	return metadataArray(messageMetadata(message).mention_usernames)
		.filter((value): value is string => typeof value === 'string' && value.trim().length > 0)
		.slice(0, 5)
}

export function messageLinkPreview(message: WhatsAppPanelMessage): { title: string | null; url: string | null; site: string | null } | null {
	const preview = metadataRecord(messageMetadata(message).whatsapp_link_preview)
	if (!preview) return null
	return {
		title: metadataString(preview.title),
		url: metadataString(preview.url),
		site: metadataString(preview.site_name ?? preview.site),
	}
}

export function messagePollSummary(message: WhatsAppPanelMessage, t: Translate): string | null {
	const poll = metadataRecord(messageMetadata(message).whatsapp_poll)
	if (!poll) return null
	const title = metadataString(poll.question ?? poll.title)
	const options = metadataArray(poll.options).length
	if (title && options) return `${title} · ${options} ${t('options')}`
	if (title) return title
	if (options) return `${options} ${t('options')}`
	return t('Poll attached')
}

export function messageLocationSummary(message: WhatsAppPanelMessage, t: Translate): string | null {
	const location = metadataRecord(messageMetadata(message).whatsapp_location)
	if (!location) return null
	const label = metadataString(location.label ?? location.name ?? location.address)
	const lat = typeof location.latitude === 'number' ? location.latitude : null
	const lon = typeof location.longitude === 'number' ? location.longitude : null
	if (label && lat != null && lon != null) return `${label} · ${lat}, ${lon}`
	if (label) return label
	if (lat != null && lon != null) return `${lat}, ${lon}`
	return t('Shared location')
}

export function messageContactCardSummary(message: WhatsAppPanelMessage, t: Translate): string | null {
	const card = metadataRecord(messageMetadata(message).whatsapp_contact_card)
	if (!card) return null
	const displayName = metadataString(card.display_name ?? card.name)
	const phones = metadataArray(card.phones)
		.map((entry) => metadataString(metadataRecord(entry)?.value ?? entry))
		.filter((value): value is string => Boolean(value))
	if (displayName && phones[0]) return `${displayName} · ${phones[0]}`
	return displayName ?? phones[0] ?? t('Contact card attached')
}

export function messageStickerSummary(message: WhatsAppPanelMessage, t: Translate): string | null {
	const sticker = metadataRecord(messageMetadata(message).whatsapp_sticker)
	if (!sticker) return null
	return metadataString(sticker.emoji ?? sticker.label ?? sticker.pack_name) ?? t('Sticker attached')
}

export function messageSystemSummary(message: WhatsAppPanelMessage, t: Translate): string | null {
	const metadata = messageMetadata(message)
	const systemMessage = metadataRecord(metadata.whatsapp_system_message)
	if (systemMessage) {
		return metadataString(systemMessage.text ?? systemMessage.kind ?? systemMessage.type) ?? t('System message')
	}
	const joinLeave = metadataRecord(metadata.whatsapp_join_leave)
	if (joinLeave) {
		return metadataString(joinLeave.text ?? joinLeave.action ?? joinLeave.kind) ?? t('Membership update')
	}
	return null
}

export function reactionSummary(message: WhatsAppPanelMessage): TelegramReactionGroup[] {
	const summary = messageMetadata(message).reaction_summary
	if (!summary || typeof summary !== 'object' || !Array.isArray((summary as { reactions?: unknown[] }).reactions)) {
		return []
	}
	return (summary as { reactions: unknown[] }).reactions
		.filter(
			(item): item is { reaction?: unknown; reaction_emoji?: unknown; count?: unknown } =>
				typeof item === 'object' && item !== null
		)
		.map((item) => ({
			reaction_emoji:
				typeof item.reaction_emoji === 'string'
					? item.reaction_emoji
					: typeof item.reaction === 'string'
						? item.reaction
						: '',
			count: typeof item.count === 'number' ? item.count : 1,
			senders: [],
		}))
		.filter((item) => item.reaction_emoji)
}
```

### `frontend/src/domains/communications/queries/aiReplyVariants.boundary.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/queries/aiReplyVariants.boundary.test.ts`
- Size bytes / Размер в байтах: `606`
- Included characters / Включено символов: `606`
- Truncated / Обрезано: `no`

```typescript
import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('AI reply variants query boundary', () => {
  it('wraps reply variants behind a TanStack mutation', () => {
    const source = readFileSync(new URL('./mailActionQueries.ts', import.meta.url), 'utf8')

    expect(source).toContain('generateAiReplyVariants')
    expect(source).toContain('useGenerateAiReplyVariantsMutation')
    expect(source).toContain('AiReplyVariantsResponse')
    expect(source).toContain('languages')
    expect(source).toContain('tones')
    expect(source).not.toContain('fetch(')
  })
})
```

### `frontend/src/domains/communications/queries/attachmentTranslationMutation.boundary.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/queries/attachmentTranslationMutation.boundary.test.ts`
- Size bytes / Размер в байтах: `672`
- Included characters / Включено символов: `672`
- Truncated / Обрезано: `no`

```typescript
import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('attachment translation mutation boundary', () => {
  it('routes attachment translation through TanStack mutation and the communications API client', () => {
    const source = readFileSync(new URL('./mailWorkspaceQueries.ts', import.meta.url), 'utf8')

    expect(source).toContain('translateAttachment')
    expect(source).toContain('AttachmentTranslationResponse')
    expect(source).toContain('export function useTranslateAttachmentMutation()')
    expect(source).toContain('useMutation<')
    expect(source).toContain('translateAttachment(attachmentId, request)')
  })
})
```

### `frontend/src/domains/communications/queries/callQueries.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/queries/callQueries.ts`
- Size bytes / Размер в байтах: `1562`
- Included characters / Включено символов: `1562`
- Truncated / Обрезано: `no`

```typescript
import { useQuery } from '@tanstack/vue-query'
import { computed, toValue } from 'vue'
import {
  fetchProviderCalls,
  fetchProviderCallTranscript,
} from '../api/communications'
import type {
  ProviderCall,
  ProviderCallTranscript,
} from '../types/communications'
import {
  communicationDetailQueryOptions,
  communicationRealtimeQueryOptions,
} from './communicationQueryPolicies'
import type { NullableQueryParam, QueryParam } from './queryTypes'

export function useProviderCallsQuery(
  accountId?: QueryParam<string>,
  limit: QueryParam<number> = 50,
  provider?: QueryParam<string>
) {
  return useQuery<ProviderCall[]>({
    queryKey: computed(() => [
      'communications-calls',
      toValue(accountId),
      toValue(limit),
      toValue(provider),
    ]),
    queryFn: async () => {
      const response = await fetchProviderCalls(
        toValue(accountId),
        toValue(limit),
        toValue(provider)
      )
      return response.items
    },
    ...communicationRealtimeQueryOptions,
  })
}

export function useProviderCallTranscriptQuery(
  callId: NullableQueryParam<string>
) {
  return useQuery<ProviderCallTranscript | null>({
    queryKey: computed(() => ['communications-call-transcript', toValue(callId)]),
    queryFn: async () => {
      const currentCallId = toValue(callId)?.trim() ?? ''
      if (!currentCallId) return null
      return (await fetchProviderCallTranscript(currentCallId)).transcript
    },
    enabled: computed(() => Boolean(toValue(callId)?.trim())),
    ...communicationDetailQueryOptions,
  })
}
```

### `frontend/src/domains/communications/queries/communicationPrefetch.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/queries/communicationPrefetch.test.ts`
- Size bytes / Размер в байтах: `6567`
- Included characters / Включено символов: `6567`
- Truncated / Обрезано: `no`

```typescript
import { QueryClient } from '@tanstack/vue-query'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { fetchCommunicationMessage, fetchCommunicationMessages, fetchThreadMessages } from '../api/communications'
import type { CommunicationMessageDetailResponse, CommunicationMessagesResponse, ThreadMessagesResponse } from '../types/communications'
import type { AttachmentSearchResult } from '../types/attachments'
import type { CommunicationSavedSearch } from '../types/savedSearches'
import {
  communicationListQueryKey,
  communicationMessageQueryKey,
  prefetchCommunicationMessageForAttachmentResult,
  prefetchCommunicationListForSavedSearch,
  prefetchCommunicationMessage,
  prefetchThreadMessages,
  threadMessagesQueryKey
} from './communicationPrefetch'

vi.mock('../api/communications', () => ({
  fetchCommunicationMessage: vi.fn(),
  fetchCommunicationMessages: vi.fn(),
  fetchThreadMessages: vi.fn()
}))

const fetchCommunicationMessageMock = vi.mocked(fetchCommunicationMessage)
const fetchCommunicationMessagesMock = vi.mocked(fetchCommunicationMessages)
const fetchThreadMessagesMock = vi.mocked(fetchThreadMessages)

function messageDetail(messageId: string): CommunicationMessageDetailResponse {
  return {
    message: {
      message_id: messageId,
      raw_record_id: 'raw-1',
      account_id: 'account-1',
      provider_record_id: 'provider-1',
      subject: 'Quarterly update',
      sender: 'sender@example.com',
      recipients: ['recipient@example.com'],
      body_text: 'Full body',
      body_html: null,
      occurred_at: '2026-06-14T10:00:00Z',
      projected_at: '2026-06-14T10:01:00Z',
      channel_kind: 'email',
      conversation_id: 'thread-1',
      sender_display_name: 'Sender',
      delivery_state: 'delivered',
      workflow_state: 'new',
      importance_score: null,
      ai_category: null,
      ai_summary: null,
      ai_summary_generated_at: null,
      message_metadata: {},
      local_state: 'active',
      local_state_changed_at: null,
      local_state_reason: null
    },
    attachments: []
  }
}

function queryClient() {
  return new QueryClient({
    defaultOptions: {
      queries: {
        retry: false
      }
    }
  })
}

function threadMessages(): ThreadMessagesResponse {
  return {
    items: []
  }
}

function communicationMessages(): CommunicationMessagesResponse {
  return {
    items: [],
    next_cursor: null,
    has_more: false
  }
}

function savedSearch(overrides: Partial<CommunicationSavedSearch> = {}): CommunicationSavedSearch {
  return {
    saved_search_id: 'search-1',
    name: 'Needs reply',
    description: null,
    account_id: 'account-1',
    query: 'quarterly',
    workflow_state: 'needs_action',
    local_state: 'active',
    channel_kind: 'email',
    is_smart_folder: false,
    sort_order: 0,
    message_count: 2,
    created_at: '2026-06-15T10:00:00Z',
    updated_at: '2026-06-15T10:00:00Z',
    ...overrides
  }
}

function attachmentSearchResult(overrides: Partial<AttachmentSearchResult> = {}): AttachmentSearchResult {
  return {
    attachment_id: 'attachment-1',
    message_id: 'msg-attachment-1',
    raw_record_id: 'raw-1',
    account_id: 'account-1',
    message_subject: 'Quarterly report',
    sender: 'sender@example.com',
    occurred_at: '2026-06-14T10:00:00Z',
    blob_id: 'blob-1',
    provider_attachment_id: 'provider-attachment-1',
    filename: 'report.pdf',
    content_type: 'application/pdf',
    size_bytes: 1024,
    sha256: 'hash-1',
    disposition: 'attachment',
    scan_status: 'not_scanned',
    scan_engine: null,
    scan_checked_at: null,
    scan_summary: null,
    storage_kind: 'local_blob',
    storage_path: 'mail/blob-1',
    created_at: '2026-06-14T10:00:00Z',
    updated_at: '2026-06-14T10:00:00Z',
    ...overrides
  }
}

describe('communication prefetch query helpers', () => {
  beforeEach(() => {
    fetchCommunicationMessageMock.mockReset()
    fetchCommunicationMessagesMock.mockReset()
    fetchThreadMessagesMock.mockReset()
  })

  it('prefetches message detail into the TanStack Query cache', async () => {
    const client = queryClient()
    const detail = messageDetail('msg-1')
    fetchCommunicationMessageMock.mockResolvedValueOnce(detail)

    await prefetchCommunicationMessage(client, ' msg-1 ')

    expect(fetchCommunicationMessageMock).toHaveBeenCalledWith('msg-1')
    expect(client.getQueryData(communicationMessageQueryKey('msg-1'))).toEqual(detail)
  })

  it('ignores blank message ids', async () => {
    const client = queryClient()

    await prefetchCommunicationMessage(client, '  ')

    expect(fetchCommunicationMessageMock).not.toHaveBeenCalled()
  })

  it('prefetches thread messages into the shared TanStack Query cache', async () => {
    const client = queryClient()
    const response = threadMessages()
    fetchThreadMessagesMock.mockResolvedValueOnce(response)

    await prefetchThreadMessages(client, ' account-1 ', ' Quarterly update ')

    expect(fetchThreadMessagesMock).toHaveBeenCalledWith('account-1', 'Quarterly update', 100)
    expect(client.getQueryData(threadMessagesQueryKey('account-1', 'Quarterly update'))).toEqual(response)
  })

  it('ignores blank thread prefetch inputs', async () => {
    const client = queryClient()

    await prefetchThreadMessages(client, 'account-1', '  ')

    expect(fetchThreadMessagesMock).not.toHaveBeenCalled()
  })

  it('prefetches the first communication list page for a saved search', async () => {
    const client = queryClient()
    const response = communicationMessages()
    fetchCommunicationMessagesMock.mockResolvedValueOnce(response)

    await prefetchCommunicationListForSavedSearch(client, savedSearch(), 'fallback-account')

    expect(fetchCommunicationMessagesMock).toHaveBeenCalledWith(
      'account-1',
      'needs_action',
      'email',
      'quarterly',
      'active',
      250,
      null
    )
    expect(client.getQueryData(communicationListQueryKey('account-1', 'needs_action', 'email', 'quarterly', 'active'))).toEqual(response)
  })

  it('prefetches the parent message for an attachment search result', async () => {
    const client = queryClient()
    const detail = messageDetail('msg-attachment-1')
    fetchCommunicationMessageMock.mockResolvedValueOnce(detail)

    await prefetchCommunicationMessageForAttachmentResult(client, attachmentSearchResult())

    expect(fetchCommunicationMessageMock).toHaveBeenCalledWith('msg-attachment-1')
    expect(client.getQueryData(communicationMessageQueryKey('msg-attachment-1'))).toEqual(detail)
  })
})
```

### `frontend/src/domains/communications/queries/communicationPrefetch.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/queries/communicationPrefetch.ts`
- Size bytes / Размер в байтах: `4115`
- Included characters / Включено символов: `4115`
- Truncated / Обрезано: `no`

```typescript
import { useQueryClient, type QueryClient } from '@tanstack/vue-query'
import { fetchCommunicationMessage, fetchCommunicationMessages, fetchThreadMessages } from '../api/communications'
import type { LocalMessageState, CommunicationMessageDetailResponse, CommunicationMessagesResponse, ThreadMessagesResponse, WorkflowState } from '../types/communications'
import type { AttachmentSearchResult } from '../types/attachments'
import type { CommunicationSavedSearch } from '../types/savedSearches'

const MESSAGE_PREFETCH_STALE_MS = 30_000
const THREAD_MESSAGES_PREFETCH_STALE_MS = 30_000

export function communicationMessageQueryKey(messageId: string) {
  return ['communications-message', messageId] as const
}

export function communicationListQueryKey(
  accountId?: string,
  workflowState?: WorkflowState | '',
  channelKind?: string,
  query?: string,
  localState?: LocalMessageState
) {
  return [
    'communications-list',
    accountId,
    workflowState,
    channelKind,
    query,
    localState
  ] as const
}

export function threadMessagesQueryKey(accountId: string, subject: string) {
  return ['communications-thread-messages', accountId, subject] as const
}

export async function prefetchCommunicationMessage(
  queryClient: QueryClient,
  messageId: string
): Promise<void> {
  const normalizedMessageId = messageId.trim()
  if (!normalizedMessageId) return

  await queryClient.prefetchQuery<CommunicationMessageDetailResponse>({
    queryKey: communicationMessageQueryKey(normalizedMessageId),
    queryFn: () => fetchCommunicationMessage(normalizedMessageId),
    staleTime: MESSAGE_PREFETCH_STALE_MS
  })
}

export function useCommunicationMessagePrefetch() {
  const queryClient = useQueryClient()
  return (messageId: string) => prefetchCommunicationMessage(queryClient, messageId)
}

export async function prefetchCommunicationMessageForAttachmentResult(
  queryClient: QueryClient,
  result: AttachmentSearchResult
): Promise<void> {
  await prefetchCommunicationMessage(queryClient, result.message_id)
}

export function useAttachmentSearchResultPrefetch() {
  const queryClient = useQueryClient()
  return (result: AttachmentSearchResult) => prefetchCommunicationMessageForAttachmentResult(queryClient, result)
}

export async function prefetchThreadMessages(
  queryClient: QueryClient,
  accountId: string,
  subject: string
): Promise<void> {
  const normalizedAccountId = accountId.trim()
  const normalizedSubject = subject.trim()
  if (!normalizedAccountId || !normalizedSubject) return

  await queryClient.prefetchQuery<ThreadMessagesResponse>({
    queryKey: threadMessagesQueryKey(normalizedAccountId, normalizedSubject),
    queryFn: () => fetchThreadMessages(normalizedAccountId, normalizedSubject, 100),
    staleTime: THREAD_MESSAGES_PREFETCH_STALE_MS
  })
}

export function useThreadMessagesPrefetch() {
  const queryClient = useQueryClient()
  return (accountId: string, subject: string) => prefetchThreadMessages(queryClient, accountId, subject)
}

export async function prefetchCommunicationListForSavedSearch(
  queryClient: QueryClient,
  savedSearch: CommunicationSavedSearch,
  fallbackAccountId?: string | null
): Promise<void> {
  const accountId = savedSearch.account_id?.trim() || fallbackAccountId?.trim() || undefined
  const workflowState = savedSearch.workflow_state ?? undefined
  const channelKind = savedSearch.channel_kind?.trim() || undefined
  const query = savedSearch.query.trim() || undefined
  const localState = savedSearch.local_state

  await queryClient.prefetchQuery<CommunicationMessagesResponse>({
    queryKey: communicationListQueryKey(accountId, workflowState, channelKind, query, localState),
    queryFn: () => fetchCommunicationMessages(accountId, workflowState, channelKind, query, localState, 250, null),
    staleTime: MESSAGE_PREFETCH_STALE_MS
  })
}

export function useSavedSearchCommunicationListPrefetch(accountId: () => string | null | undefined) {
  const queryClient = useQueryClient()
  return (savedSearch: CommunicationSavedSearch) => prefetchCommunicationListForSavedSearch(queryClient, savedSearch, accountId())
}
```

### `frontend/src/domains/communications/queries/communicationQueryPolicies.boundary.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/queries/communicationQueryPolicies.boundary.test.ts`
- Size bytes / Размер в байтах: `1437`
- Included characters / Включено символов: `1437`
- Truncated / Обрезано: `no`

```typescript
import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('Communication query policies', () => {
  it('defines explicit background refetch policies for communication queries', () => {
    const source = readFileSync(new URL('./communicationQueryPolicies.ts', import.meta.url), 'utf8')

    expect(source).toContain('communicationRealtimeQueryOptions')
    expect(source).toContain('communicationDetailQueryOptions')
    expect(source).toContain('communicationReferenceQueryOptions')
    expect(source).toContain('refetchOnReconnect')
    expect(source).toContain('refetchOnWindowFocus')
    expect(source).toContain('refetchInterval')
  })

  it('applies shared policies from domain query hooks', () => {
    const core = readFileSync(new URL('./mailCoreQueries.ts', import.meta.url), 'utf8')
    const workspace = readFileSync(new URL('./mailWorkspaceQueries.ts', import.meta.url), 'utf8')
    const operations = readFileSync(new URL('./mailOperationQueries.ts', import.meta.url), 'utf8')

    expect(core).toContain('communicationRealtimeQueryOptions')
    expect(core).toContain('communicationDetailQueryOptions')
    expect(core).toContain('communicationReferenceQueryOptions')
    expect(workspace).toContain('communicationRealtimeQueryOptions')
    expect(workspace).toContain('communicationReferenceQueryOptions')
    expect(operations).toContain('communicationRealtimeQueryOptions')
  })
})
```

### `frontend/src/domains/communications/queries/communicationQueryPolicies.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/queries/communicationQueryPolicies.ts`
- Size bytes / Размер в байтах: `509`
- Included characters / Включено символов: `509`
- Truncated / Обрезано: `no`

```typescript
const SECOND_MS = 1000

export const communicationRealtimeQueryOptions = {
  staleTime: 10 * SECOND_MS,
  refetchInterval: 60 * SECOND_MS,
  refetchOnReconnect: true,
  refetchOnWindowFocus: true
} as const

export const communicationDetailQueryOptions = {
  staleTime: 30 * SECOND_MS,
  refetchOnReconnect: true,
  refetchOnWindowFocus: true
} as const

export const communicationReferenceQueryOptions = {
  staleTime: 5 * 60 * SECOND_MS,
  refetchOnReconnect: true,
  refetchOnWindowFocus: false
} as const
```

### `frontend/src/domains/communications/queries/draftsInfiniteQuery.boundary.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/queries/draftsInfiniteQuery.boundary.test.ts`
- Size bytes / Размер в байтах: `816`
- Included characters / Включено символов: `816`
- Truncated / Обрезано: `no`

```typescript
import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('drafts infinite query boundary', () => {
  it('uses TanStack infinite query cursor loading for drafts', () => {
    const source = readFileSync(new URL('./mailOperationQueries.ts', import.meta.url), 'utf8')

    expect(source).toContain('useInfiniteQuery<DraftListResponse')
    expect(source).toContain("queryKey: computed(() => ['communications-drafts', toValue(accountId)]")
    expect(source).toContain('initialPageParam: null')
    expect(source).toContain('fetchDrafts(toValue(accountId), undefined, 50, pageParam)')
    expect(source).toContain('getNextPageParam: (lastPage) => lastPage.next_cursor ?? undefined')
    expect(source).toContain('select: (data) => data.pages.flatMap((page) => page.items)')
  })
})
```

### `frontend/src/domains/communications/queries/folderMailList.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/queries/folderMailList.test.ts`
- Size bytes / Размер в байтах: `1407`
- Included characters / Включено символов: `1407`
- Truncated / Обрезано: `no`

```typescript
import { describe, expect, it } from 'vitest'
import type { FolderMessage } from '../types/folders'
import { folderMessagesToMailSummaries } from './folderMailList'

function folderMessage(overrides: Partial<FolderMessage> = {}): FolderMessage {
  return {
    folder_id: 'folder-1',
    message_id: 'msg-1',
    account_id: 'account-1',
    subject: 'Quarterly update',
    sender: 'alice@example.com',
    occurred_at: null,
    projected_at: '2026-06-15T10:00:00Z',
    workflow_state: 'waiting',
    local_state: 'active',
    added_at: '2026-06-15T10:05:00Z',
    attachment_count: 3,
    ...overrides
  }
}

describe('folder mail list mapping', () => {
  it('maps cursor-paginated folder message rows into existing mail list summaries', () => {
    const [summary] = folderMessagesToMailSummaries([folderMessage()])

    expect(summary).toMatchObject({
      message_id: 'msg-1',
      account_id: 'account-1',
      subject: 'Quarterly update',
      sender: 'alice@example.com',
      occurred_at: null,
      projected_at: '2026-06-15T10:00:00Z',
      channel_kind: 'email',
      workflow_state: 'waiting',
      local_state: 'active',
      attachment_count: 3
    })
    expect(summary.recipients).toEqual([])
    expect(summary.body_text_preview).toBe('')
    expect(summary.message_metadata).toEqual({
      folder_id: 'folder-1',
      folder_added_at: '2026-06-15T10:05:00Z'
    })
  })
})
```

### `frontend/src/domains/communications/queries/folderMailList.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/queries/folderMailList.ts`
- Size bytes / Размер в байтах: `1865`
- Included characters / Включено символов: `1865`
- Truncated / Обрезано: `no`

```typescript
import { computed, toValue, type MaybeRefOrGetter } from 'vue'
import { useFolderMessagesQuery } from './useCommunicationsQuery'
import type { CommunicationMessageSummary } from '../types/communications'
import type { FolderMessage } from '../types/folders'

export function folderMessagesToMailSummaries(
  folderMessages: FolderMessage[]
): CommunicationMessageSummary[] {
  return folderMessages.map((message) => ({
    message_id: message.message_id,
    raw_record_id: '',
    account_id: message.account_id,
    provider_record_id: '',
    subject: message.subject,
    sender: message.sender,
    recipients: [],
    body_text_preview: '',
    occurred_at: message.occurred_at,
    projected_at: message.projected_at,
    channel_kind: 'email',
    conversation_id: null,
    sender_display_name: null,
    delivery_state: 'folder',
    workflow_state: message.workflow_state,
    importance_score: null,
    ai_category: null,
    ai_summary: null,
    ai_summary_generated_at: null,
    message_metadata: {
      folder_id: message.folder_id,
      folder_added_at: message.added_at
    },
    attachment_count: message.attachment_count,
    local_state: message.local_state,
    local_state_changed_at: null
  }))
}

export function useFolderMailList(folderId: MaybeRefOrGetter<string | null | undefined>) {
  const activeFolderId = computed(() => toValue(folderId)?.trim() || null)
  const query = useFolderMessagesQuery(
    () => activeFolderId.value,
    () => Boolean(activeFolderId.value)
  )
  const messages = computed(() => folderMessagesToMailSummaries(query.data.value ?? []))
  const errorMessage = computed(() => {
    if (!query.error.value) return ''
    return query.error.value instanceof Error
      ? query.error.value.message
      : 'Folder messages request failed'
  })

  return {
    ...query,
    messages,
    errorMessage
  }
}
```

### `frontend/src/domains/communications/queries/mailActionQueries.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/queries/mailActionQueries.ts`
- Size bytes / Размер в байтах: `8536`
- Included characters / Включено символов: `8536`
- Truncated / Обрезано: `no`

```typescript
import { useMutation, useQueryClient } from '@tanstack/vue-query'
import {
  analyzeMessage,
  addMessageLabel,
  bulkMessageAction,
  exportMessage,
  deleteMessageFromProvider,
  markMessageRead,
  extractMessageNotes,
  extractMessageTasks,
  fetchMessageAuth,
  fetchMessageExplain,
  fetchMessageSignature,
  fetchMessageSmartCc,
  detectMessageLanguage,
  generateAiReply,
  generateAiReplyVariants,
  runMailFullResync,
  runMailSyncNow,
  toggleMessageImportant,
  toggleMessageMute,
  toggleMessagePin,
  snoozeMessage,
  translateThread,
  translateMessage
} from '../api/communications'
import type {
  BulkMessageActionResponse,
  ExtractNotesResponse,
  ExtractTasksResponse,
  MailSyncRunResponse,
  MessageAnalyzeResponse,
  AiReplyResponse,
  AiReplyVariantsResponse,
  LanguageDetection,
  MessageAuthCheckResponse,
  MessageExplainResponse,
  MessageExportResponse,
  MessageExportFormat,
  MessageImportantToggleResponse,
  MessagePinToggleResponse,
  LocalMessageStateResponse,
  SignatureDetection,
  SmartCcResponse,
  TranslationResponse
} from '../types/communications'
import type { ThreadTranslationResponse } from '../types/multilingual'

function invalidateMessageViews(queryClient: ReturnType<typeof useQueryClient>, messageId: string) {
  queryClient.invalidateQueries({ queryKey: ['communications-message', messageId] })
  queryClient.invalidateQueries({ queryKey: ['communications-list'] })
}

function invalidateSyncViews(queryClient: ReturnType<typeof useQueryClient>) {
  queryClient.invalidateQueries({ queryKey: ['communications-list'] })
  queryClient.invalidateQueries({ queryKey: ['communications-state-counts'] })
  queryClient.invalidateQueries({ queryKey: ['communications', 'mail', 'sync-statuses'] })
  queryClient.invalidateQueries({ queryKey: ['communications', 'mail', 'mailbox-health'] })
}

export function useToggleMessagePinMutation() {
  const queryClient = useQueryClient()
  return useMutation<MessagePinToggleResponse, Error, string>({
    mutationFn: async (messageId) => toggleMessagePin(messageId),
    onSuccess: (_result, messageId) => invalidateMessageViews(queryClient, messageId)
  })
}

export function useToggleMessageImportantMutation() {
  const queryClient = useQueryClient()
  return useMutation<MessageImportantToggleResponse, Error, string>({
    mutationFn: async (messageId) => toggleMessageImportant(messageId),
    onSuccess: (_result, messageId) => invalidateMessageViews(queryClient, messageId)
  })
}

export function useToggleMessageMuteMutation() {
  const queryClient = useQueryClient()
  return useMutation<MessagePinToggleResponse, Error, string>({
    mutationFn: async (messageId) => toggleMessageMute(messageId),
    onSuccess: (_result, messageId) => invalidateMessageViews(queryClient, messageId)
  })
}

export function useExportMessageMutation() {
  return useMutation<MessageExportResponse, Error, { messageId: string; format: MessageExportFormat }>({
    mutationFn: async ({ messageId, format }) => exportMessage(messageId, format)
  })
}

export function useMarkMessageReadMutation() {
  const queryClient = useQueryClient()
  return useMutation<Record<string, unknown>, Error, string>({
    mutationFn: async (messageId) => markMessageRead(messageId),
    onSuccess: (_result, messageId) => invalidateMessageViews(queryClient, messageId)
  })
}

export function useDeleteMessageFromProviderMutation() {
  const queryClient = useQueryClient()
  return useMutation<LocalMessageStateResponse, Error, string>({
    mutationFn: async (messageId) => deleteMessageFromProvider(messageId),
    onSuccess: (_result, messageId) => invalidateMessageViews(queryClient, messageId)
  })
}

export function useMarkMessageUnreadMutation() {
  const queryClient = useQueryClient()
  return useMutation<
    BulkMessageActionResponse,
    Error,
    string
  >({
    mutationFn: async (messageId) => bulkMessageAction({ action: 'mark_unread', message_ids: [messageId] }),
    onSuccess: (_result, messageId) => invalidateMessageViews(queryClient, messageId)
  })
}

export function useAddMessageLabelMutation() {
  const queryClient = useQueryClient()
  return useMutation<Record<string, unknown>, Error, { messageId: string; label: string }>({
    mutationFn: async ({ messageId, label }) => addMessageLabel(messageId, label),
    onSuccess: (_result, { messageId }) => invalidateMessageViews(queryClient, messageId)
  })
}

export function useRemoveMessageLabelMutation() {
  const queryClient = useQueryClient()
  return useMutation<BulkMessageActionResponse, Error, { messageId: string; label: string }>({
    mutationFn: async ({ messageId, label }) =>
      bulkMessageAction({ action: 'remove_label', message_ids: [messageId], label }),
    onSuccess: (_result, { messageId }) => invalidateMessageViews(queryClient, messageId)
  })
}

export function useSnoozeMessageMutation() {
  const queryClient = useQueryClient()
  return useMutation<Record<string, unknown>, Error, { messageId: string; until: string }>({
    mutationFn: async ({ messageId, until }) => snoozeMessage(messageId, until),
    onSuccess: (_result, { messageId }) => invalidateMessageViews(queryClient, messageId)
  })
}

export function useAnalyzeMessageMutation() {
  const queryClient = useQueryClient()
  return useMutation<MessageAnalyzeResponse, Error, string>({
    mutationFn: async (messageId) => analyzeMessage(messageId),
    onSuccess: (_result, messageId) => invalidateMessageViews(queryClient, messageId)
  })
}

export function useGenerateAiReplyMutation() {
  return useMutation<AiReplyResponse, Error, { messageId: string; tone: string; language: string }>({
    mutationFn: async ({ messageId, tone, language }) => generateAiReply(messageId, { tone, language })
  })
}

export function useGenerateAiReplyVariantsMutation() {
  return useMutation<
    AiReplyVariantsResponse,
    Error,
    { messageId: string; languages: string[]; tones: string[] }
  >({
    mutationFn: async ({ messageId, languages, tones }) =>
      generateAiReplyVariants(messageId, { languages, tones })
  })
}

export function useExplainMessageMutation() {
  return useMutation<MessageExplainResponse, Error, string>({
    mutationFn: async (messageId) => fetchMessageExplain(messageId)
  })
}

export function useDetectMessageLanguageMutation() {
  return useMutation<LanguageDetection, Error, string>({
    mutationFn: async (messageId) => detectMessageLanguage(messageId)
  })
}

export function useReviewMessageSecurityMutation() {
  return useMutation<
    { auth: MessageAuthCheckResponse; signature: SignatureDetection },
    Error,
    string
  >({
    mutationFn: async (messageId) => {
      const [auth, signature] = await Promise.all([
        fetchMessageAuth(messageId),
        fetchMessageSignature(messageId)
      ])
      return { auth, signature }
    }
  })
}

export function useReviewMessageRecipientsMutation() {
  return useMutation<SmartCcResponse, Error, string>({
    mutationFn: async (messageId) => fetchMessageSmartCc(messageId)
  })
}

export function useTranslateMessageMutation() {
  return useMutation<TranslationResponse, Error, { messageId: string; targetLanguage: string }>({
    mutationFn: async ({ messageId, targetLanguage }) => translateMessage(messageId, targetLanguage)
  })
}

export function useTranslateThreadMutation() {
  return useMutation<
    ThreadTranslationResponse,
    Error,
    { accountId: string; subject: string; targetLanguage: string; limit?: number }
  >({
    mutationFn: async ({ accountId, subject, targetLanguage, limit }) =>
      translateThread(accountId, subject, targetLanguage, limit)
  })
}

export function useExtractMessageTasksMutation() {
  return useMutation<ExtractTasksResponse, Error, string>({
    mutationFn: async (messageId) => extractMessageTasks(messageId)
  })
}

export function useExtractMessageNotesMutation() {
  return useMutation<ExtractNotesResponse, Error, string>({
    mutationFn: async (messageId) => extractMessageNotes(messageId)
  })
}

export function useRunMailSyncNowMutation() {
  const queryClient = useQueryClient()
  return useMutation<MailSyncRunResponse, Error, string>({
    mutationFn: async (accountId) => runMailSyncNow(accountId),
    onSuccess: () => invalidateSyncViews(queryClient)
  })
}

export function useRunMailFullResyncMutation() {
  const queryClient = useQueryClient()
  return useMutation<MailSyncRunResponse, Error, string>({
    mutationFn: async (accountId) => runMailFullResync(accountId),
    onSuccess: () => invalidateSyncViews(queryClient)
  })
}
```

### `frontend/src/domains/communications/queries/mailCertificates.boundary.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/queries/mailCertificates.boundary.test.ts`
- Size bytes / Размер в байтах: `803`
- Included characters / Включено символов: `803`
- Truncated / Обрезано: `no`

```typescript
import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('mail certificate query boundary', () => {
  it('wraps certificate API through TanStack Query hooks', () => {
    const source = readFileSync(new URL('./mailWorkspaceQueries.ts', import.meta.url), 'utf8')

    expect(source).toContain('useMailCertificatesQuery')
    expect(source).toContain('useExpiringMailCertificatesQuery')
    expect(source).toContain('useCreateMailCertificateMutation')
    expect(source).toContain('fetchMailCertificates')
    expect(source).toContain('fetchExpiringMailCertificates')
    expect(source).toContain('createMailCertificate')
    expect(source).toContain("['communications-certificates']")
    expect(source).toContain("['communications-certificates-expiring'")
  })

})
```

### `frontend/src/domains/communications/queries/mailCoreQueries.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/queries/mailCoreQueries.ts`
- Size bytes / Размер в байтах: `7486`
- Included characters / Включено символов: `7486`
- Truncated / Обрезано: `no`

```typescript
import { useInfiniteQuery, useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, toValue } from 'vue'
import {
  fetchCommunicationMessage,
  fetchCommunicationMessages,
  fetchMailSyncSettings,
  fetchMailboxHealth,
  fetchMailSyncStatus,
  fetchMessageStateCounts,
  fetchPersonas,
  fetchThreadMessages,
  fetchThreads,
  updateMailSyncSettings
} from '../api/communications'
import { fetchMessageAiState } from '../api/aiState'
import type {
  CommunicationMessageSummary,
  CommunicationPersona,
  LocalMessageState,
  MailboxHealth,
  CommunicationMessageDetailResponse,
  CommunicationMessagesResponse,
  MailSyncSettings,
  MailSyncSettingsUpdate,
  MailSyncStatus,
  CommunicationThread,
  ThreadMessagesResponse,
  ThreadListResponse,
  WorkflowState,
  WorkflowStateCountItem
} from '../types/communications'
import type { CommunicationAiStateRecord } from '../types/aiState'
import { communicationListQueryKey, communicationMessageQueryKey, threadMessagesQueryKey } from './communicationPrefetch'
import {
  communicationDetailQueryOptions,
  communicationRealtimeQueryOptions,
  communicationReferenceQueryOptions
} from './communicationQueryPolicies'
import type { NullableQueryParam, QueryParam } from './queryTypes'

export function useMailListQuery(
  accountId?: QueryParam<string>,
  workflowState?: QueryParam<WorkflowState>,
  channelKind?: QueryParam<string>,
  query?: QueryParam<string>,
  localState?: QueryParam<LocalMessageState>
) {
  return useInfiniteQuery<CommunicationMessagesResponse, Error, CommunicationMessageSummary[], readonly unknown[], string | null>({
    queryKey: computed(() => communicationListQueryKey(
      toValue(accountId),
      toValue(workflowState),
      toValue(channelKind),
      toValue(query),
      toValue(localState)
    )),
    initialPageParam: null,
    queryFn: async ({ pageParam }) => {
      return fetchCommunicationMessages(
        toValue(accountId),
        toValue(workflowState),
        toValue(channelKind),
        toValue(query),
        toValue(localState),
        250,
        pageParam
      )
    },
    getNextPageParam: (lastPage) => lastPage.next_cursor ?? undefined,
    select: (data) => {
      return data.pages.flatMap((page) => page.items)
    },
    ...communicationRealtimeQueryOptions
  })
}

export function useMessageQuery(messageId: NullableQueryParam<string>) {
  return useQuery<CommunicationMessageDetailResponse | null>({
    queryKey: computed(() => {
      const id = toValue(messageId)
      return id ? communicationMessageQueryKey(id) : ['communications-message', null] as const
    }),
    queryFn: async () => {
      const id = toValue(messageId)
      if (!id) return null
      return fetchCommunicationMessage(id)
    },
    enabled: computed(() => !!toValue(messageId)),
    ...communicationDetailQueryOptions
  })
}

export function useMessageAiStateQuery(messageId: NullableQueryParam<string>) {
  return useQuery<CommunicationAiStateRecord | null>({
    queryKey: computed(() => {
      const id = toValue(messageId)
      return id ? ['communications-ai-state', id] as const : ['communications-ai-state', null] as const
    }),
    queryFn: async () => {
      const id = toValue(messageId)
      if (!id) return null
      return fetchMessageAiState(id)
    },
    enabled: computed(() => !!toValue(messageId)),
    ...communicationRealtimeQueryOptions
  })
}

export function useStateCountsQuery(accountId?: QueryParam<string>, localState?: QueryParam<LocalMessageState>) {
  return useQuery<WorkflowStateCountItem[]>({
    queryKey: computed(() => ['communications-state-counts', toValue(accountId), toValue(localState)]),
    queryFn: async () => {
      const res = await fetchMessageStateCounts(toValue(accountId), toValue(localState))
      return res.counts
    },
    ...communicationRealtimeQueryOptions
  })
}

export function useSyncStatusesQuery() {
  return useQuery<MailSyncStatus[]>({
    queryKey: ['communications', 'mail', 'sync-statuses'],
    queryFn: async () => {
      const res = await fetchMailSyncStatus()
      return res.items
    },
    ...communicationRealtimeQueryOptions
  })
}

export function useMailSyncSettingsQuery(accountId: NullableQueryParam<string>) {
  return useQuery<MailSyncSettings | null>({
    queryKey: computed(() => {
      const id = toValue(accountId)
      return id
        ? ['communications', 'mail', 'sync-settings', id] as const
        : ['communications', 'mail', 'sync-settings', null] as const
    }),
    queryFn: async () => {
      const id = toValue(accountId)
      if (!id) return null
      return fetchMailSyncSettings(id)
    },
    enabled: computed(() => Boolean(toValue(accountId))),
    ...communicationReferenceQueryOptions
  })
}

export function useUpdateMailSyncSettingsMutation() {
  const queryClient = useQueryClient()
  return useMutation<
    MailSyncSettings,
    Error,
    { accountId: string; settings: MailSyncSettingsUpdate }
  >({
    mutationFn: async ({ accountId, settings }) => updateMailSyncSettings(accountId, settings),
    onSuccess: (_settings, variables) => {
      queryClient.invalidateQueries({ queryKey: ['communications', 'mail', 'sync-settings', variables.accountId] })
      queryClient.invalidateQueries({ queryKey: ['communications', 'mail', 'sync-statuses'] })
    }
  })
}

export function useMailboxHealthQuery(accountId?: QueryParam<string>) {
  return useQuery<MailboxHealth | null>({
    queryKey: computed(() => ['communications', 'mail', 'mailbox-health', toValue(accountId)]),
    queryFn: async () => {
      return fetchMailboxHealth(toValue(accountId))
    },
    ...communicationRealtimeQueryOptions
  })
}

export function useConversationsQuery(accountId?: QueryParam<string>) {
  return useInfiniteQuery<ThreadListResponse, Error, CommunicationThread[], readonly unknown[], string | null>({
    queryKey: computed(() => ['communications-threads', toValue(accountId)]),
    initialPageParam: null,
    queryFn: async ({ pageParam }) => fetchThreads(toValue(accountId), 50, pageParam),
    getNextPageParam: (lastPage) => lastPage.next_cursor ?? undefined,
    select: (data) => data.pages.flatMap((page) => page.items),
    ...communicationRealtimeQueryOptions
  })
}

export function useThreadMessagesQuery(accountId: NullableQueryParam<string>, subject: NullableQueryParam<string>) {
  return useQuery<ThreadMessagesResponse>({
    queryKey: computed(() => {
      const currentAccountId = toValue(accountId)?.trim() ?? ''
      const currentSubject = toValue(subject)?.trim() ?? ''
      return currentAccountId && currentSubject
        ? threadMessagesQueryKey(currentAccountId, currentSubject)
        : ['communications-thread-messages', currentAccountId, currentSubject] as const
    }),
    queryFn: async () => {
      const currentAccountId = toValue(accountId)?.trim() ?? ''
      const currentSubject = toValue(subject)?.trim() ?? ''
      if (!currentAccountId || !currentSubject) return { items: [] }
      return fetchThreadMessages(currentAccountId, currentSubject, 100)
    },
    enabled: computed(() => Boolean(toValue(accountId)?.trim() && toValue(subject)?.trim())),
    ...communicationDetailQueryOptions
  })
}

export function usePersonasQuery() {
  return useQuery<CommunicationPersona[]>({
    queryKey: ['communications-personas'],
    queryFn: async () => {
      const res = await fetchPersonas()
      return res.items
    },
    ...communicationReferenceQueryOptions
  })
}
```
