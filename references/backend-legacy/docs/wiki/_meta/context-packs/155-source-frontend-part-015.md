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

- Chunk ID / ID чанка: `155-source-frontend-part-015`
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

### `frontend/src/platform/bootstrap/realtimeTelegramCachePatches.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/platform/bootstrap/realtimeTelegramCachePatches.test.ts`
- Size bytes / Размер в байтах: `24632`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```typescript
import { describe, expect, it, vi } from 'vitest'
import { handleRealtimeEvent } from './realtime'

describe('telegram realtime cache patch handling', () => {
  it('patches cached telegram chats for typing changed events', () => {
    const chatsKey = ['communications', 'telegram', 'chats', 'account-1', 50]
    const chatDetailKey = ['communications', 'telegram', 'chat-detail', 'tgchat-1']
    const chat = {
      telegram_chat_id: 'tgchat-1',
      account_id: 'account-1',
      provider_chat_id: 'chat-1',
      chat_kind: 'private',
      title: 'Chat',
      username: null,
      sync_state: 'synced',
      last_message_at: null,
      metadata: {},
      created_at: '2026-06-16T09:00:00Z',
      updated_at: '2026-06-16T09:00:00Z'
    }
    const setQueryData = vi.fn((queryKey, updater) => {
      if (typeof updater !== 'function') return updater
      if (JSON.stringify(queryKey) === JSON.stringify(chatsKey)) return updater([chat])
      if (JSON.stringify(queryKey) === JSON.stringify(chatDetailKey)) return updater(chat)
      return updater(undefined)
    })
    const queryClient = {
      invalidateQueries: vi.fn(),
      getQueriesData: vi.fn().mockImplementation(({ queryKey }) => {
        const key = JSON.stringify(queryKey)
        if (key === JSON.stringify(['communications', 'telegram', 'chats'])) return [[chatsKey, [chat]]]
        if (key === JSON.stringify(['communications', 'telegram', 'chat-detail'])) return [[chatDetailKey, chat]]
        return []
      }),
      setQueryData
    }

    handleRealtimeEvent(
      {
        id: 'tg-56',
        event: 'event',
        data: JSON.stringify({
          event: {
            event_type: 'telegram.typing.changed',
            occurred_at: '2026-06-16T09:00:00.000Z',
            payload: {
              telegram_chat_id: 'tgchat-1',
              provider_chat_id: 'chat-1',
              sender_id: 'user:777',
              action: 'chatActionTyping',
              is_active: true
            }
          }
        })
      },
      queryClient
    )

    expect(setQueryData.mock.results[0]?.value[0].metadata.active_typing).toMatchObject({
      sender_id: 'user:777',
      action: 'chatActionTyping',
      is_active: true,
      expires_at: '2026-06-16T09:00:07.000Z'
    })
    expect(setQueryData.mock.results[1]?.value.metadata.active_typing.sender_id).toBe('user:777')
  })

  it('patches cached telegram chat detail and list snapshots for provider unread progress updates', () => {
    const chatsKey = ['communications', 'telegram', 'chats', 'account-1', 50]
    const chatDetailKey = ['communications', 'telegram', 'chat-detail', 'tgchat-1']
    const chat = {
      telegram_chat_id: 'tgchat-1',
      account_id: 'account-1',
      provider_chat_id: 'chat-1',
      chat_kind: 'private',
      title: 'Chat',
      username: null,
      sync_state: 'synced',
      last_message_at: null,
      metadata: { unread_count: 4 },
      created_at: '2026-06-16T09:00:00Z',
      updated_at: '2026-06-16T09:00:00Z'
    }
    const updatedChat = {
      ...chat,
      metadata: {
        ...chat.metadata,
        unread_count: 1,
        provider_unread_count: 1,
        last_read_inbox_provider_message_id: '777',
      },
    }
    const setQueryData = vi.fn((queryKey, updater) => {
      if (typeof updater !== 'function') return updater
      if (JSON.stringify(queryKey) === JSON.stringify(chatsKey)) return updater([chat])
      if (JSON.stringify(queryKey) === JSON.stringify(chatDetailKey)) return updater(chat)
      return updater(undefined)
    })
    const queryClient = {
      invalidateQueries: vi.fn(),
      getQueriesData: vi.fn().mockImplementation(({ queryKey }) => {
        const key = JSON.stringify(queryKey)
        if (key === JSON.stringify(['communications', 'telegram', 'chats'])) return [[chatsKey, [chat]]]
        if (key === JSON.stringify(['communications', 'telegram', 'chat-detail'])) return [[chatDetailKey, chat]]
        return []
      }),
      setQueryData
    }

    handleRealtimeEvent(
      {
        id: 'tg-56b',
        event: 'event',
        data: JSON.stringify({
          event: {
            event_type: 'telegram.chat.updated',
            occurred_at: '2026-06-16T09:00:00.000Z',
            payload: {
              telegram_chat_id: 'tgchat-1',
              provider_chat_id: 'chat-1',
              chat: updatedChat
            }
          }
        })
      },
      queryClient
    )

    expect(setQueryData.mock.results[0]?.value[0].metadata.last_read_inbox_provider_message_id).toBe('777')
    expect(setQueryData.mock.results[1]?.value.metadata.last_read_inbox_provider_message_id).toBe('777')
  })

  it('patches cached telegram folder filters for provider folder update events', () => {
    const foldersKey = ['communications', 'telegram', 'folders', 'account-1']
    const folders = [
      { id: 'local:all', label: 'All', source: 'local', count: 2, icon: 'tabler:message' },
      { id: 'folder:Work', label: 'Work', source: 'telegram', count: 2, icon: 'tabler:folder', provider_folder_id: 7 },
    ]
    const setQueryData = vi.fn((queryKey, updater) =>
      typeof updater === 'function' ? updater(folders) : updater
    )
    const queryClient = {
      invalidateQueries: vi.fn(),
      getQueriesData: vi.fn().mockImplementation(({ queryKey }) => {
        const key = JSON.stringify(queryKey)
        if (key === JSON.stringify(['communications', 'telegram', 'folders'])) return [[foldersKey, folders]]
        return []
      }),
      setQueryData
    }

    handleRealtimeEvent(
      {
        id: 'tg-folders-1',
        event: 'event',
        data: JSON.stringify({
          event: {
            event_type: 'telegram.folders.updated',
            occurred_at: '2026-06-17T10:00:00.000Z',
            payload: {
              account_id: 'account-1',
              items: [
                { id: 'local:all', label: 'All', source: 'local', count: 3, icon: 'tabler:message' },
                { id: 'folder:Projects', label: 'Projects', source: 'telegram', count: 2, icon: 'tabler:folder', provider_folder_id: 9 },
              ],
            },
            metadata: {
              account_id: 'account-1',
            }
          }
        })
      },
      queryClient
    )

    expect(setQueryData.mock.results[0]?.value).toEqual([
      { id: 'local:all', label: 'All', source: 'local', count: 3, icon: 'tabler:message', provider_folder_id: null },
      { id: 'folder:Projects', label: 'Projects', source: 'telegram', count: 2, icon: 'tabler:folder', provider_folder_id: 9 },
    ])
  })


  it('patches cached telegram message reaction summary for telegram reaction events', () => {
    const messageKey = ['communications', 'telegram', 'messages', 'account-1', 'chat-1', 50]
    const messages = [
      {
        message_id: 'tg-msg-1',
        raw_record_id: 'raw-1',
        account_id: 'account-1',
        provider_message_id: 'provider-1',
        provider_chat_id: 'chat-1',
        chat_title: 'Chat',
        sender: 'sender-1',
        sender_display_name: 'Sender',
        text: 'Hello',
        occurred_at: '2026-06-16T09:00:00Z',
        projected_at: '2026-06-16T09:00:01Z',
        channel_kind: 'telegram_user',
        delivery_state: 'received',
        metadata: {}
      }
    ]
    const setQueryData = vi.fn((queryKey, updater) =>
      typeof updater === 'function' ? updater(messages) : updater
    )
    const queryClient = {
      invalidateQueries: vi.fn(),
      getQueriesData: vi.fn().mockReturnValue([[messageKey, messages]]),
      setQueryData
    }

    handleRealtimeEvent(
      {
        id: 'tg-57',
        event: 'event',
        data: JSON.stringify({
          event: {
            event_type: 'telegram.reaction.changed',
            subject: { id: 'tg-msg-1', kind: 'telegram_message' },
            payload: {
              reaction_emoji: '👍',
              is_active: true
            }
          }
        })
      },
      queryClient
    )

    const patchedItems = setQueryData.mock.results[0]?.value
    expect(patchedItems[0].metadata.reaction_summary.reactions[0]).toMatchObject({
      reaction_emoji: '👍',
      count: 1
    })
  })

  it('patches cached telegram lifecycle metadata for telegram message updated events', () => {
    const messageKey = ['communications', 'telegram', 'messages', 'account-1', 'chat-1', 50]
    const pinnedKey = ['communications', 'telegram', 'chats', 'tgchat-1', 'pinned-messages', 100]
    const messages = [
      {
        message_id: 'tg-msg-2',
        raw_record_id: 'raw-2',
        account_id: 'account-1',
        provider_message_id: 'provider-2',
        provider_chat_id: 'chat-1',
        chat_title: 'Chat',
        sender: 'sender-2',
        sender_display_name: 'Sender',
        text: 'Hello again',
        occurred_at: '2026-06-16T09:05:00Z',
        projected_at: '2026-06-16T09:05:01Z',
        channel_kind: 'telegram_user',
        delivery_state: 'received',
        metadata: {}
      }
    ]
    const pinnedResponse = { items: [] }
    const searchKey = ['communications', 'telegram', 'search', 'messages', 'hello', 'account-1', 'chat-1', 50]
    const searchResponse = { query: 'hello', items: [], total: 0 }
    const setQueryData = vi.fn((queryKey, updater) => {
      if (typeof updater !== 'function') return updater
      if (JSON.stringify(queryKey) === JSON.stringify(messageKey)) return updater(messages)
      if (JSON.stringify(queryKey) === JSON.stringify(pinnedKey)) return updater(pinnedResponse)
      if (JSON.stringify(queryKey) === JSON.stringify(searchKey)) return updater(searchResponse)
      return updater(undefined)
    })
    const updatedSnapshot = {
      ...messages[0],
      text: 'Hello again',
      metadata: { is_pinned: true, pinned: true }
    }
    const queryClient = {
      invalidateQueries: vi.fn(),
      getQueriesData: vi.fn().mockImplementation(({ queryKey }) => {
        const key = JSON.stringify(queryKey)
        if (key === JSON.stringify(['communications', 'telegram', 'messages'])) {
          return [[messageKey, messages]]
        }
        if (key === JSON.stringify(['communications', 'telegram', 'chats'])) {
          return [[pinnedKey, pinnedResponse]]
        }
        if (key === JSON.stringify(['communications', 'telegram', 'search', 'messages'])) {
          return [[searchKey, searchResponse]]
        }
        return []
      }),
      setQueryData
    }

    handleRealtimeEvent(
      {
        id: 'tg-58',
        event: 'event',
        data: JSON.stringify({
          event: {
            event_type: 'telegram.message.updated',
            subject: { id: 'tg-msg-2', kind: 'telegram_message' },
            payload: {
              version_number: 3,
              is_pinned: true,
              telegram_chat_id: 'tgchat-1',
              message: updatedSnapshot
            }
          }
        })
      },
      queryClient
    )

    const patchedItems = setQueryData.mock.results[0]?.value
    expect(patchedItems[0].metadata.lifecycle.latest_version_number).toBe(3)
    expect(patchedItems[0].metadata.is_pinned).toBe(true)

    const patchedPinned = setQueryData.mock.results[1]?.value
    expect(patchedPinned.items[0].message_id).toBe('tg-msg-2')

    const patchedSearch = setQueryData.mock.results[2]?.value
    expect(patchedSearch.items[0].message_id).toBe('tg-msg-2')
    expect(patchedSearch.total).toBe(1)
  })

  it('upserts telegram message snapshots for telegram created events', () => {
    const messageKey = ['communications', 'telegram', 'messages', 'account-1', 'chat-1', 50]
    const messages = [
      {
        message_id: 'tg-msg-1',
        raw_record_id: 'raw-1',
        account_id: 'account-1',
        provider_message_id: 'provider-1',
        provider_chat_id: 'chat-1',
        chat_title: 'Chat',
        sender: 'sender-1',
        sender_display_name: 'Sender',
        text: 'Older message',
        occurred_at: '2026-06-16T09:00:00Z',
        project
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `frontend/src/platform/bootstrap/realtimeTelegramCommandPatches.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/platform/bootstrap/realtimeTelegramCommandPatches.test.ts`
- Size bytes / Размер в байтах: `21934`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```typescript
import { describe, expect, it, vi } from 'vitest'
import { handleRealtimeEvent } from './realtime'

describe('telegram command realtime cache patch handling', () => {
  it('patches cached telegram command rows for retry scheduling and dead-letter fields', () => {
    const commandsKey = ['integrations', 'telegram', 'commands', 'account-1']
    const commands = [
      {
        command_id: 'cmd-retry-1',
        account_id: 'account-1',
        command_kind: 'send_media',
        idempotency_key: 'idem-retry-1',
        provider_chat_id: 'chat-1',
        provider_message_id: null,
        target_ref: {},
        payload: {},
        capability_state: 'available',
        action_class: 'provider_write',
        confirmation_decision: 'not_required',
        status: 'executing',
        retry_count: 1,
        max_retries: 3,
        last_error: null,
        result_payload: {},
        audit_metadata: {},
        actor_id: 'makosh-frontend',
        happened_at: '2026-06-17T09:00:00Z',
        next_attempt_at: null,
        last_attempt_at: '2026-06-17T09:00:00Z',
        locked_at: null,
        locked_by: null,
        provider_observed_at: null,
        provider_state: {},
        reconciliation_status: 'awaiting_provider',
        reconciled_at: null,
        dead_lettered_at: null,
        completed_at: null,
        created_at: '2026-06-17T09:00:00Z',
        updated_at: '2026-06-17T09:00:00Z'
      }
    ]
    const setQueryData = vi.fn((queryKey, updater) =>
      typeof updater === 'function' ? updater(commands) : updater
    )
    const queryClient = {
      invalidateQueries: vi.fn(),
      getQueriesData: vi.fn().mockImplementation(({ queryKey }) => {
        if (JSON.stringify(queryKey) === JSON.stringify(['integrations', 'telegram', 'commands'])) {
          return [[commandsKey, commands]]
        }
        return []
      }),
      setQueryData
    }

    handleRealtimeEvent(
      {
        id: 'tg-command-retrying',
        event: 'event',
        data: JSON.stringify({
          event: {
            event_type: 'telegram.command.status_changed',
            metadata: { account_id: 'account-1' },
            payload: {
              command_id: 'cmd-retry-1',
              status: 'retrying',
              retry_count: 2,
              last_error: 'temporary tdlib failure',
              next_attempt_at: '2026-06-17T09:01:30Z'
            }
          }
        })
      },
      queryClient
    )

    const retryingCommands = setQueryData.mock.results[0]?.value
    expect(retryingCommands[0]).toMatchObject({
      status: 'retrying',
      retry_count: 2,
      last_error: 'temporary tdlib failure',
      next_attempt_at: '2026-06-17T09:01:30Z'
    })

    handleRealtimeEvent(
      {
        id: 'tg-command-dead-letter',
        event: 'event',
        data: JSON.stringify({
          event: {
            event_type: 'telegram.command.status_changed',
            metadata: { account_id: 'account-1' },
            payload: {
              command_id: 'cmd-retry-1',
              status: 'dead_letter',
              retry_count: 3,
              last_error: 'permanent tdlib failure',
              dead_lettered_at: '2026-06-17T09:02:00Z'
            }
          }
        })
      },
      queryClient
    )

    const deadLetteredCommands = setQueryData.mock.results[1]?.value
    expect(deadLetteredCommands[0]).toMatchObject({
      status: 'dead_letter',
      retry_count: 3,
      last_error: 'permanent tdlib failure',
      dead_lettered_at: '2026-06-17T09:02:00Z'
    })
  })

  it('patches cached telegram command rows for provider reconciliation events', () => {
    const commandsKey = ['integrations', 'telegram', 'commands', 'account-1']
    const commands = [
      {
        command_id: 'cmd-reconciled-1',
        account_id: 'account-1',
        command_kind: 'edit',
        idempotency_key: 'idem-1',
        provider_chat_id: 'chat-1',
        provider_message_id: 'provider-msg-1',
        target_ref: {},
        payload: {},
        capability_state: 'available',
        action_class: 'provider_write',
        confirmation_decision: 'not_required',
        status: 'executing',
        retry_count: 1,
        max_retries: 3,
        last_error: null,
        result_payload: {},
        audit_metadata: {},
        actor_id: 'makosh-frontend',
        happened_at: '2026-06-17T09:00:00Z',
        next_attempt_at: null,
        last_attempt_at: '2026-06-17T09:00:00Z',
        locked_at: null,
        locked_by: null,
        provider_observed_at: null,
        provider_state: {},
        reconciliation_status: 'awaiting_provider',
        reconciled_at: null,
        dead_lettered_at: null,
        completed_at: null,
        created_at: '2026-06-17T09:00:00Z',
        updated_at: '2026-06-17T09:00:00Z'
      }
    ]
    const setQueryData = vi.fn((queryKey, updater) =>
      typeof updater === 'function' ? updater(commands) : updater
    )
    const queryClient = {
      invalidateQueries: vi.fn(),
      getQueriesData: vi.fn().mockImplementation(({ queryKey }) => {
        if (JSON.stringify(queryKey) === JSON.stringify(['integrations', 'telegram', 'commands'])) {
          return [[commandsKey, commands]]
        }
        return []
      }),
      setQueryData
    }

    handleRealtimeEvent(
      {
        id: 'tg-command-reconciled',
        event: 'event',
        data: JSON.stringify({
          event: {
            event_type: 'telegram.command.reconciled',
            metadata: { account_id: 'account-1' },
            payload: {
              command_id: 'cmd-reconciled-1',
              status: 'completed',
              retry_count: 1,
              provider_chat_id: 'chat-1',
              message_id: 'provider-msg-1',
              provider_observed_at: '2026-06-17T09:00:05Z',
              provider_state: {
                source_event: 'tdlib.updateMessageContent'
              },
              result_payload: {
                projection_message_id: 'msg-1'
              },
              reconciliation_status: 'observed',
              reconciled_at: '2026-06-17T09:00:05Z',
              completed_at: '2026-06-17T09:00:05Z'
            }
          }
        })
      },
      queryClient
    )

    const patchedCommands = setQueryData.mock.results[0]?.value
    expect(patchedCommands[0].status).toBe('completed')
    expect(patchedCommands[0].retry_count).toBe(1)
    expect(patchedCommands[0].reconciliation_status).toBe('observed')
    expect(patchedCommands[0].provider_observed_at).toBe('2026-06-17T09:00:05Z')
    expect(patchedCommands[0].reconciled_at).toBe('2026-06-17T09:00:05Z')
    expect(patchedCommands[0].completed_at).toBe('2026-06-17T09:00:05Z')
    expect(patchedCommands[0].provider_state).toMatchObject({
      source_event: 'tdlib.updateMessageContent'
    })
    expect(patchedCommands[0].result_payload).toMatchObject({
      projection_message_id: 'msg-1'
    })
  })

  it('patches cached telegram command rows for provider mismatch reconciliation events', () => {
    const commandsKey = ['integrations', 'telegram', 'commands', 'account-1']
    const commands = [
      {
        command_id: 'cmd-edit-mismatch-1',
        account_id: 'account-1',
        command_kind: 'edit',
        idempotency_key: 'idem-edit-mismatch-1',
        provider_chat_id: 'chat-1',
        provider_message_id: 'provider-msg-1',
        target_ref: {},
        payload: {
          new_text: 'Expected provider edit body'
        },
        capability_state: 'available',
        action_class: 'provider_write',
        confirmation_decision: 'not_required',
        status: 'executing',
        retry_count: 1,
        max_retries: 3,
        last_error: null,
        result_payload: {},
        audit_metadata: {},
        actor_id: 'makosh-frontend',
        happened_at: '2026-06-17T09:00:00Z',
        next_attempt_at: null,
        last_attempt_at: '2026-06-17T09:00:00Z',
        locked_at: null,
        locked_by: null,
        provider_observed_at: null,
        provider_state: {},
        reconciliation_status: 'awaiting_provider',
        reconciled_at: null,
        dead_lettered_at: null,
        completed_at: null,
        created_at: '2026-06-17T09:00:00Z',
        updated_at: '2026-06-17T09:00:00Z'
      }
    ]
    const setQueryData = vi.fn((queryKey, updater) =>
      typeof updater === 'function' ? updater(commands) : updater
    )
    const queryClient = {
      invalidateQueries: vi.fn(),
      getQueriesData: vi.fn().mockImplementation(({ queryKey }) => {
        if (JSON.stringify(queryKey) === JSON.stringify(['integrations', 'telegram', 'commands'])) {
          return [[commandsKey, commands]]
        }
        return []
      }),
      setQueryData
    }

    handleRealtimeEvent(
      {
        id: 'tg-command-mismatch',
        event: 'event',
        data: JSON.stringify({
          event: {
            event_type: 'telegram.command.reconciled',
            metadata: { account_id: 'account-1' },
            payload: {
              command_id: 'cmd-edit-mismatch-1',
              status: 'failed',
              retry_count: 1,
              provider_chat_id: 'chat-1',
              provider_message_id: 'provider-msg-1',
              provider_observed_at: '2026-06-17T09:00:05Z',
              provider_state: {
                expected_body_text: 'Expected provider edit body',
                observed_body_text: 'Observed provider body'
              },
              result_payload: {
                expected_body_text: 'Expected provider edit body',
                observed_body_text: 'Observed provider body',
                mismatch: true
              },
              last_error: 'Provider observed a different message body than requested',
              reconciliation_status: 'mismatch',
              reconciled_at: '2026-06-17T09:00:05Z',
              completed_at: null
            }
          }
        })
      },
      queryClient
    )

    const patchedCommands = setQueryData.mock.results[0]?.value
    expect(patchedCommands[0].status).toBe('failed')
    expect(patchedCommands[0].reconciliation_status).toBe('mismatch')
    expect(patchedCommands[0].last_error).toBe(
      'Provider observed a different message body than requested'
    )
    expect(patchedCommands[0].provider_state).toMatchObject({
      expected_body_text: 'Expected provider edit body',
      observed_body_text: 'Observed provider body'
    })
    expect(patchedCommands[0].result_payload).toMatchObject({
      expected_body_text: 'Expected provider edit body',
      observed_body_text: 'Observed provider body',
      mismatch: true
    })
    expect(patchedCommands[0].completed_at).toBeNull()
    expect(patchedCommands[0].reconciled_at).toBe('2026-06-17T09:00:05Z')
  })

  it('inserts a queued send_media command row when media upload starts before command query refetch', () => {
    const commandsKey = ['integrations', 'telegram', 'commands', 'account-1', 20]
    const commands: Array<Record<string, unknown>> = []
    const setQueryData = vi.fn((queryKey, updater) =>
      typeof updater === 'function' ? updater(commands) : updater
    )
    const queryClient = {
      invalidateQueries: vi.fn(),
      getQueriesData: vi.fn().mockImplementation(({ queryKey }) => {
        if (JSON.stringify(queryKey) === JSON.stringify(['integrations', 'telegram', 'commands'])) {
          return [[commandsKey, commands]]
        }
        return []
      }),
      setQueryData
    }

    handleRealtimeEvent(
      {
        id: 'tg-upload-started',
        event: 'event',
        data: JSON.stringify({
          event: {
            event_type: 'telegram.media.upload.started',
            payload: {
              command_id: 'cmd-upload-1',
              account_id: 'account-1',
              provider_chat_id: 'chat-1',
              command_kind: 'send_media',
              idempotency_key: 'idem-upload-1',
              capability_state: 'available',
              act
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `frontend/src/platform/bootstrap/realtimeTelegramCommandQueryFilters.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/platform/bootstrap/realtimeTelegramCommandQueryFilters.test.ts`
- Size bytes / Размер в байтах: `2281`
- Included characters / Включено символов: `2281`
- Truncated / Обрезано: `no`

```typescript
import { describe, expect, it, vi } from 'vitest'
import { handleRealtimeEvent } from './realtime'

describe('telegram command realtime query filters', () => {
  it('inserts command rows only into matching filtered command caches', () => {
    const matchingKey = ['integrations', 'telegram', 'commands', 'account-1', 20, 'chat-1', 'chat-1:42', 'mark_read|mark_unread']
    const otherChatKey = ['integrations', 'telegram', 'commands', 'account-1', 20, 'chat-2', 'chat-1:42', 'mark_read|mark_unread']
    const otherMessageKey = ['integrations', 'telegram', 'commands', 'account-1', 20, 'chat-1', 'chat-1:99', 'mark_read|mark_unread']
    const otherKindKey = ['integrations', 'telegram', 'commands', 'account-1', 20, 'chat-1', 'chat-1:42', 'join|leave']
    const commands: Array<Record<string, unknown>> = []

    const setQueryData = vi.fn((queryKey, updater) =>
      typeof updater === 'function' ? updater(commands) : updater
    )
    const queryClient = {
      invalidateQueries: vi.fn(),
      getQueriesData: vi.fn().mockImplementation(({ queryKey }) => {
        if (JSON.stringify(queryKey) === JSON.stringify(['integrations', 'telegram', 'commands'])) {
          return [
            [matchingKey, commands],
            [otherChatKey, commands],
            [otherMessageKey, commands],
            [otherKindKey, commands],
          ]
        }
        return []
      }),
      setQueryData,
    }

    handleRealtimeEvent(
      {
        id: 'tg-command-filter-1',
        event: 'event',
        data: JSON.stringify({
          event: {
            event_type: 'telegram.command.status_changed',
            payload: {
              command_id: 'cmd-read-1',
              account_id: 'account-1',
              provider_chat_id: 'chat-1',
              provider_message_id: 'chat-1:42',
              command_kind: 'mark_read',
              status: 'queued',
              retry_count: 0,
              max_retries: 3,
              capability_state: 'available',
              action_class: 'provider_write',
              confirmation_decision: 'confirmed',
            },
          },
        }),
      },
      queryClient
    )

    expect(setQueryData).toHaveBeenCalledTimes(1)
    expect(setQueryData.mock.results[0]?.value).toHaveLength(1)
  })
})
```

### `frontend/src/platform/bootstrap/realtimeTelegramInvalidation.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/platform/bootstrap/realtimeTelegramInvalidation.test.ts`
- Size bytes / Размер в байтах: `6314`
- Included characters / Включено символов: `6314`
- Truncated / Обрезано: `no`

```typescript
import { describe, expect, it, vi } from 'vitest'
import { handleRealtimeEvent } from './realtime'

describe('telegram realtime invalidation handling', () => {
  it('invalidates telegram chat and message queries for telegram message events', () => {
    const queryClient = { invalidateQueries: vi.fn() }

    handleRealtimeEvent(
      {
        id: 'tg-45',
        event: 'event',
        data: JSON.stringify({
          event: {
            event_type: 'telegram.message.created',
          },
        }),
      },
      queryClient
    )

    expect(queryClient.invalidateQueries).toHaveBeenCalledTimes(2)
    expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
      queryKey: ['communications', 'telegram', 'messages'],
    })
    expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
      queryKey: ['communications', 'telegram', 'chats'],
    })
  })

  it('invalidates telegram runtime-related queries for telegram sync progress events', () => {
    const queryClient = { invalidateQueries: vi.fn() }

    handleRealtimeEvent(
      {
        id: 'tg-46',
        event: 'event',
        data: JSON.stringify({
          event: {
            event_type: 'telegram.sync.progress',
          },
        }),
      },
      queryClient
    )

    expect(queryClient.invalidateQueries).toHaveBeenCalledTimes(3)
    expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
      queryKey: ['communications', 'telegram', 'chats'],
    })
    expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
      queryKey: ['communications', 'telegram', 'messages'],
    })
    expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
      queryKey: ['integrations', 'telegram', 'runtime'],
    })
  })

  it('invalidates telegram message and runtime queries for command status events', () => {
    const queryClient = { invalidateQueries: vi.fn() }

    handleRealtimeEvent(
      {
        id: 'tg-47',
        event: 'event',
        data: JSON.stringify({
          event: {
            event_type: 'telegram.command.status_changed',
          },
        }),
      },
      queryClient
    )

    expect(queryClient.invalidateQueries).toHaveBeenCalledTimes(3)
    expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
      queryKey: ['communications', 'telegram', 'messages'],
    })
    expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
      queryKey: ['integrations', 'telegram', 'runtime'],
    })
    expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
      queryKey: ['integrations', 'telegram', 'commands'],
    })
  })

  it('invalidates telegram message and media search queries for media events', () => {
    const queryClient = { invalidateQueries: vi.fn() }

    handleRealtimeEvent(
      {
        id: 'tg-48',
        event: 'event',
        data: JSON.stringify({
          event: {
            event_type: 'telegram.media.download.progress',
          },
        }),
      },
      queryClient
    )

    expect(queryClient.invalidateQueries).toHaveBeenCalledTimes(2)
    expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
      queryKey: ['communications', 'telegram', 'messages'],
    })
    expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
      queryKey: ['communications', 'telegram', 'search', 'media'],
    })
  })

  it('invalidates telegram command queue queries for media upload events', () => {
    const queryClient = { invalidateQueries: vi.fn() }

    handleRealtimeEvent(
      {
        id: 'tg-48-upload',
        event: 'event',
        data: JSON.stringify({
          event: {
            event_type: 'telegram.media.upload.failed',
          },
        }),
      },
      queryClient
    )

    expect(queryClient.invalidateQueries).toHaveBeenCalledTimes(2)
    expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
      queryKey: ['integrations', 'telegram', 'commands'],
    })
    expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
      queryKey: ['integrations', 'telegram', 'runtime'],
    })
  })

  it('invalidates telegram chat and runtime queries for typing events', () => {
    const queryClient = { invalidateQueries: vi.fn() }

    handleRealtimeEvent(
      {
        id: 'tg-49',
        event: 'event',
        data: JSON.stringify({
          event: {
            event_type: 'telegram.typing.changed',
          },
        }),
      },
      queryClient
    )

    expect(queryClient.invalidateQueries).toHaveBeenCalledTimes(2)
    expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
      queryKey: ['communications', 'telegram', 'chats'],
    })
    expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
      queryKey: ['integrations', 'telegram', 'runtime'],
    })
  })

  it('invalidates telegram member queries for participant updates', () => {
    const queryClient = { invalidateQueries: vi.fn() }

    handleRealtimeEvent(
      {
        id: 'tg-50',
        event: 'event',
        data: JSON.stringify({
          event: {
            event_type: 'telegram.participant.updated',
          },
        }),
      },
      queryClient
    )

    expect(queryClient.invalidateQueries).toHaveBeenCalledTimes(2)
    expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
      queryKey: ['communications', 'telegram', 'chat-members'],
    })
    expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
      queryKey: ['communications', 'telegram', 'chats'],
    })
  })

  it('invalidates telegram folder and chat queries for provider chat updates', () => {
    const queryClient = { invalidateQueries: vi.fn() }

    handleRealtimeEvent(
      {
        id: 'tg-folder-membership-1',
        event: 'event',
        data: JSON.stringify({
          event: {
            event_type: 'telegram.chat.updated',
            payload: {
              action: 'provider_chat_position_update',
              list_kind: 'folder',
              provider_folder_id: 9,
              order: 42,
            },
          },
        }),
      },
      queryClient
    )

    expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
      queryKey: ['communications', 'telegram', 'folders'],
    })
    expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
      queryKey: ['communications', 'telegram', 'chats'],
    })
  })
})
```

### `frontend/src/platform/bootstrap/realtimeTelegramMediaCachePatches.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/platform/bootstrap/realtimeTelegramMediaCachePatches.test.ts`
- Size bytes / Размер в байтах: `12220`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```typescript
import { describe, expect, it, vi } from 'vitest'
import { handleRealtimeEvent } from './realtime'

function applyQueryDataUpdate<TData>(
	current: TData | undefined,
	updater: TData | ((data: TData | undefined) => TData | undefined)
): TData | undefined {
	if (typeof updater !== 'function') return updater
	const applyUpdater = updater as (data: TData | undefined) => TData | undefined
	return applyUpdater(current)
}

describe('telegram media realtime cache patch handling', () => {
  it('patches cached telegram message and media search results for started and progress events', () => {
    const messageKey = ['communications', 'telegram', 'messages', 'account-1', 'chat-1', 50]
    const mediaKey = ['communications', 'telegram', 'search', 'media', '', 'account-1', 'chat-1', 'all', 100]
    const message = {
      message_id: 'tg-msg-media-1',
      raw_record_id: 'raw-media-1',
      account_id: 'account-1',
      provider_message_id: 'provider-media-1',
      provider_chat_id: 'chat-1',
      chat_title: 'Chat',
      sender: 'sender-1',
      sender_display_name: 'Sender',
      text: '',
      occurred_at: '2026-06-16T10:15:00Z',
      projected_at: '2026-06-16T10:15:01Z',
      channel_kind: 'telegram_user',
      delivery_state: 'received',
      metadata: {
        attachments: [
          {
            attachment_id: 'att-1',
            attachment_type: 'photo',
            filename: 'before.jpg',
            content_type: 'image/jpeg',
            download_state: 'remote',
          },
        ],
      },
    }
    const mediaItem = {
      message_id: 'tg-msg-media-1',
      provider_message_id: 'provider-media-1',
      provider_chat_id: 'chat-1',
      file_name: 'before.jpg',
      kind: 'photo',
      mime_type: 'image/jpeg',
      size_bytes: null,
      occurred_at: '2026-06-16T10:15:00Z',
      download_state: 'remote',
      tdlib_file_id: null,
      provider_attachment_id: 'att-1',
      local_path: null,
    }
    let currentMessages = [message]
    let currentMediaResponse = { query: '', items: [mediaItem] }
    const setQueryData = vi.fn((queryKey, updater) => {
      if (JSON.stringify(queryKey) === JSON.stringify(messageKey)) {
        currentMessages = applyQueryDataUpdate(currentMessages, updater) ?? currentMessages
        return currentMessages
      }
      if (JSON.stringify(queryKey) === JSON.stringify(mediaKey)) {
        currentMediaResponse =
          applyQueryDataUpdate(currentMediaResponse, updater) ?? currentMediaResponse
        return currentMediaResponse
      }
      return applyQueryDataUpdate(undefined, updater)
    })
    const queryClient = {
      invalidateQueries: vi.fn(),
      getQueriesData: vi.fn().mockImplementation(({ queryKey }) => {
        const key = JSON.stringify(queryKey)
        if (key === JSON.stringify(['communications', 'telegram', 'messages'])) return [[messageKey, currentMessages]]
        if (key === JSON.stringify(['communications', 'telegram', 'search', 'media'])) {
          return [[mediaKey, currentMediaResponse]]
        }
        return []
      }),
      setQueryData,
    }

    handleRealtimeEvent(
      {
        id: 'tg-media-started',
        event: 'event',
        data: JSON.stringify({
          event: {
            event_type: 'telegram.media.download.started',
            subject: { id: 'provider-media-1', kind: 'telegram_message' },
            payload: {
              provider_chat_id: 'chat-1',
              provider_message_id: 'provider-media-1',
              provider_attachment_id: 'att-1',
              tdlib_file_id: 9001,
              download_state: 'requested',
            },
          },
        }),
      },
      queryClient
    )

    handleRealtimeEvent(
      {
        id: 'tg-media-progress',
        event: 'event',
        data: JSON.stringify({
          event: {
            event_type: 'telegram.media.download.progress',
            subject: { id: 'provider-media-1', kind: 'telegram_message' },
            payload: {
              provider_chat_id: 'chat-1',
              provider_message_id: 'provider-media-1',
              provider_attachment_id: 'att-1',
              tdlib_file_id: 9001,
              download_state: 'downloading',
              expected_size_bytes: 4096,
              downloaded_size_bytes: 1024,
              is_downloading_active: true,
              is_downloading_completed: false,
            },
          },
        }),
      },
      queryClient
    )

    expect(currentMessages[0].metadata.attachments[0]).toMatchObject({
      attachment_id: 'att-1',
      tdlib_file_id: 9001,
      download_state: 'downloading',
      expected_size_bytes: 4096,
      downloaded_size_bytes: 1024,
      is_downloading_active: true,
      is_downloading_completed: false,
    })
    expect(currentMediaResponse.items[0]).toMatchObject({
      provider_attachment_id: 'att-1',
      tdlib_file_id: 9001,
      download_state: 'downloading',
    })
  })

  it('patches cached telegram message and media search results for failed download events', () => {
    const messageKey = ['communications', 'telegram', 'messages', 'account-1', 'chat-1', 50]
    const mediaKey = ['communications', 'telegram', 'search', 'media', '', 'account-1', 'chat-1', 'all', 100]
    const messages = [
      {
        message_id: 'tg-msg-media-2',
        raw_record_id: 'raw-media-2',
        account_id: 'account-1',
        provider_message_id: 'provider-media-2',
        provider_chat_id: 'chat-1',
        chat_title: 'Chat',
        sender: 'sender-1',
        sender_display_name: 'Sender',
        text: '',
        occurred_at: '2026-06-16T10:16:00Z',
        projected_at: '2026-06-16T10:16:01Z',
        channel_kind: 'telegram_user',
        delivery_state: 'received',
        metadata: {
          attachments: [
            {
              attachment_id: 'att-2',
              attachment_type: 'photo',
              filename: 'failed.jpg',
              content_type: 'image/jpeg',
              tdlib_file_id: 9002,
              download_state: 'downloading',
            },
          ],
        },
      },
    ]
    const mediaResponse = {
      query: '',
      items: [
        {
          message_id: 'tg-msg-media-2',
          provider_message_id: 'provider-media-2',
          provider_chat_id: 'chat-1',
          file_name: 'failed.jpg',
          kind: 'photo',
          mime_type: 'image/jpeg',
          size_bytes: null,
          occurred_at: '2026-06-16T10:16:00Z',
          download_state: 'downloading',
          tdlib_file_id: 9002,
          provider_attachment_id: 'att-2',
          local_path: null,
        },
      ],
    }
    const setQueryData = vi.fn((queryKey, updater) => {
      if (JSON.stringify(queryKey) === JSON.stringify(messageKey)) {
        return applyQueryDataUpdate(messages, updater)
      }
      if (JSON.stringify(queryKey) === JSON.stringify(mediaKey)) {
        return applyQueryDataUpdate(mediaResponse, updater)
      }
      return applyQueryDataUpdate(undefined, updater)
    })
    const queryClient = {
      invalidateQueries: vi.fn(),
      getQueriesData: vi.fn().mockImplementation(({ queryKey }) => {
        const key = JSON.stringify(queryKey)
        if (key === JSON.stringify(['communications', 'telegram', 'messages'])) return [[messageKey, messages]]
        if (key === JSON.stringify(['communications', 'telegram', 'search', 'media'])) {
          return [[mediaKey, mediaResponse]]
        }
        return []
      }),
      setQueryData,
    }

    handleRealtimeEvent(
      {
        id: 'tg-media-failed',
        event: 'event',
        data: JSON.stringify({
          event: {
            event_type: 'telegram.media.download.failed',
            subject: { id: 'provider-media-2', kind: 'telegram_message' },
            payload: {
              provider_chat_id: 'chat-1',
              provider_message_id: 'provider-media-2',
              provider_attachment_id: 'att-2',
              tdlib_file_id: 9002,
              download_state: 'failed',
              error: 'tdlib timeout',
            },
          },
        }),
      },
      queryClient
    )

    const patchedMessages = setQueryData.mock.results[0]?.value
    expect(patchedMessages[0].metadata.attachments[0]).toMatchObject({
      download_state: 'failed',
      last_error: 'tdlib timeout',
    })

    const patchedMedia = setQueryData.mock.results[1]?.value
    expect(patchedMedia.items[0]).toMatchObject({
      download_state: 'failed',
      tdlib_file_id: 9002,
    })
  })

  it('patches cached telegram message and media search results for completed download events', () => {
    const messageKey = ['communications', 'telegram', 'messages', 'account-1', 'chat-1', 50]
    const mediaKey = ['communications', 'telegram', 'search', 'media', '', 'account-1', 'chat-1', 'all', 100]
    const messages = [
      {
        message_id: 'tg-msg-media-3',
        raw_record_id: 'raw-media-3',
        account_id: 'account-1',
        provider_message_id: 'provider-media-3',
        provider_chat_id: 'chat-1',
        chat_title: 'Chat',
        sender: 'sender-1',
        sender_display_name: 'Sender',
        text: '',
        occurred_at: '2026-06-16T10:15:00Z',
        projected_at: '2026-06-16T10:15:01Z',
        channel_kind: 'telegram_user',
        delivery_state: 'received',
        metadata: {
          attachments: [
            {
              attachment_id: 'att-3',
              attachment_type: 'photo',
              filename: 'before.jpg',
              content_type: 'image/jpeg',
              download_state: 'remote',
            },
          ],
        },
      },
    ]
    const mediaResponse = { query: '', items: [] }
    const downloadedSnapshot = {
      ...messages[0],
      metadata: {
        attachments: [
          {
            attachment_id: 'att-3',
            attachment_type: 'photo',
            filename: 'after.jpg',
            content_type: 'image/jpeg',
            download_state: 'downloaded',
            local_path: '/tmp/after.jpg',
            size: 2048,
          },
        ],
      },
    }
    const setQueryData = vi.fn((queryKey, updater) => {
      if (JSON.stringify(queryKey) === JSON.stringify(messageKey)) {
        return applyQueryDataUpdate(messages, updater)
      }
      if (JSON.stringify(queryKey) === JSON.stringify(mediaKey)) {
        return applyQueryDataUpdate(mediaResponse, updater)
      }
      return applyQueryDataUpdate(undefined, updater)
    })
    const queryClient = {
      invalidateQueries: vi.fn(),
      getQueriesData: vi.fn().mockImplementation(({ queryKey }) => {
        const key = JSON.stringify(queryKey)
        if (key === JSON.stringify(['communications', 'telegram', 'messages'])) return [[messageKey, messages]]
        if (key === JSON.stringify(['communications', 'telegram', 'search', 'media'])) {
          return [[mediaKey, mediaResponse]]
        }
        return []
      }),
      setQueryData,
    }

    handleRealtimeEvent(
      {
        id: 'tg-media-downloaded',
        event: 'event',
        data: JSON.stringify({
          event: {
            event_type: 'telegram.media.downloaded',
            subject: { id: 'tg-msg-media-3', kind: 'telegram_message' },
            payload: {
              provider_chat_id: 'chat-1',
              provider_message_id: 'provider-media-3',
              attachment_id: 'att-3',
              download_state: 'downloaded',
              local_path: '/tmp/after.jpg',
              message: downloadedSnapshot,
            },
          },
        }),
      },
      queryClient
    )

    const patchedMessages = setQueryData.mock.results[0]?.value
    expect(patchedMessages[0].metadata.attachments[0].download_state).toBe('downloaded')
    expect(patchedMessages[0].metadata.attachments[0].local_path).toBe('/tmp/after.jpg')

    const patchedMedia = setQueryData.mock.results[1]?.value
    expect(patchedMedia.items).toHaveLength(1)
    expect(patchedMedia.items[0])
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `frontend/src/platform/bootstrap/realtimeTelegramMembersSync.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/platform/bootstrap/realtimeTelegramMembersSync.test.ts`
- Size bytes / Размер в байтах: `1939`
- Included characters / Включено символов: `1939`
- Truncated / Обрезано: `no`

```typescript
import { describe, expect, it, vi } from 'vitest'
import { handleRealtimeEvent } from './realtime'

describe('telegram members sync realtime handling', () => {
  it('patches cached telegram runtime status for members sync events', () => {
    const runtimeKey = ['integrations', 'telegram', 'runtime', 'account-1']
    const runtimeStatus = {
      account_id: 'account-1',
      provider_kind: 'telegram_user',
      runtime_kind: 'tdlib_qr_authorized',
      status: 'idle',
      fixture_runtime: false,
      tdjson_runtime_available: true,
      telegram_app_credentials_configured: true,
      live_send_available: true,
      last_error: null,
      updated_at: '2026-06-17T09:00:00Z'
    }
    const setQueryData = vi.fn((queryKey, updater) =>
      typeof updater === 'function' ? updater(runtimeStatus) : updater
    )
    const queryClient = {
      invalidateQueries: vi.fn(),
      getQueriesData: vi.fn().mockImplementation(({ queryKey }) => {
        if (JSON.stringify(queryKey) === JSON.stringify(['communications', 'telegram', 'messages'])) return []
        return [[runtimeKey, runtimeStatus]]
      }),
      setQueryData
    }

    handleRealtimeEvent(
      {
        id: 'tg-members-sync-1',
        event: 'event',
        data: JSON.stringify({
          event: {
            event_type: 'telegram.sync.completed',
            metadata: { account_id: 'account-1' },
            payload: {
              scope: 'members',
              status: 'completed',
              synced_count: 2,
              provider_chat_id: 'chat-1'
            }
          }
        })
      },
      queryClient
    )

    const patchedRuntime = setQueryData.mock.results[0]?.value
    expect(patchedRuntime.last_sync_scope).toBe('members')
    expect(patchedRuntime.last_sync_status).toBe('completed')
    expect(patchedRuntime.last_synced_count).toBe(2)
    expect(patchedRuntime.last_sync_provider_chat_id).toBe('chat-1')
  })
})
```

### `frontend/src/platform/bootstrap/realtimeTelegramParticipantPatches.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/platform/bootstrap/realtimeTelegramParticipantPatches.test.ts`
- Size bytes / Размер в байтах: `7076`
- Included characters / Включено символов: `7076`
- Truncated / Обрезано: `no`

```typescript
import { describe, expect, it, vi } from 'vitest'
import { handleRealtimeEvent } from './realtime'

describe('telegram participant realtime cache patching', () => {
  it('upserts cached chat members for participant update events across infinite member pages', () => {
    const membersKey = ['communications', 'telegram', 'chat-members', 'tgchat-1', 50, '', '']
    const existing = {
      pages: [
        {
          items: [
            {
              sender_id: 'user:1',
              sender_display_name: 'Old Member',
              message_count: 0,
              last_message_at: null,
              source: 'tdlib',
              provider_member_id: 'user:1',
              username: null,
              role: 'member',
              status: 'member',
              is_admin: false,
              is_owner: false,
              permissions: {},
              observed_at: null,
            },
          ],
          next_cursor: null,
        },
      ],
      pageParams: [null],
    }
    const setQueryData = vi.fn((queryKey, updater) =>
      typeof updater === 'function' ? updater(existing) : updater
    )
    const queryClient = {
      invalidateQueries: vi.fn(),
      getQueriesData: vi.fn().mockImplementation(({ queryKey }) => {
        if (JSON.stringify(queryKey) === JSON.stringify(['communications', 'telegram', 'chat-members'])) {
          return [[membersKey, existing]]
        }
        return []
      }),
      setQueryData
    }

    handleRealtimeEvent(
      {
        id: 'tg-participant-1',
        event: 'event',
        data: JSON.stringify({
          event: {
            event_type: 'telegram.participant.updated',
            payload: {
              telegram_chat_id: 'tgchat-1',
              participant: {
                sender_id: 'user:42',
                sender_display_name: 'Owner User',
                provider_member_id: 'user:42',
                source: 'tdlib',
                role: 'owner',
                status: 'creator',
                is_admin: true,
                is_owner: true,
                permissions: { can_invite_users: true },
                observed_at: '2026-06-17T00:00:00Z'
              }
            }
          }
        })
      },
      queryClient
    )

    const patched = setQueryData.mock.results[0]?.value
    const patchedItems = patched.pages[0].items
    expect(patchedItems[0]).toMatchObject({
      provider_member_id: 'user:42',
      sender_display_name: 'Owner User',
      role: 'owner',
      is_owner: true
    })
    expect(patchedItems[1].provider_member_id).toBe('user:1')
  })

  it('removes cached chat members when exhaustive provider absence is observed', () => {
    const membersKey = ['communications', 'telegram', 'chat-members', 'tgchat-1', 50, '', '']
    const existing = {
      pages: [
        {
          items: [
            {
              sender_id: 'user:42',
              sender_display_name: 'Owner User',
              message_count: 0,
              last_message_at: null,
              source: 'tdlib',
              provider_member_id: 'user:42',
              username: null,
              role: 'owner',
              status: 'creator',
              is_admin: true,
              is_owner: true,
              permissions: {},
              observed_at: null,
            },
          ],
          next_cursor: null,
        },
      ],
      pageParams: [null],
    }
    const setQueryData = vi.fn((queryKey, updater) =>
      typeof updater === 'function' ? updater(existing) : updater
    )
    const queryClient = {
      invalidateQueries: vi.fn(),
      getQueriesData: vi.fn().mockImplementation(({ queryKey }) => {
        if (JSON.stringify(queryKey) === JSON.stringify(['communications', 'telegram', 'chat-members'])) {
          return [[membersKey, existing]]
        }
        return []
      }),
      setQueryData
    }

    handleRealtimeEvent(
      {
        id: 'tg-participant-absence-1',
        event: 'event',
        data: JSON.stringify({
          event: {
            event_type: 'telegram.participant.updated',
            payload: {
              telegram_chat_id: 'tgchat-1',
              participant: {
                sender_id: 'user:42',
                sender_display_name: 'Owner User',
                provider_member_id: 'user:42',
                source: 'tdlib',
                role: 'owner',
                status: 'absent_exhaustive',
                is_admin: true,
                is_owner: true,
                permissions: {
                  membership_state: 'absent_exhaustive',
                },
                observed_at: '2026-06-17T00:00:00Z'
              }
            }
          }
        })
      },
      queryClient
    )

    const patched = setQueryData.mock.results[0]?.value
    expect(patched.pages[0].items).toEqual([])
  })

  it('removes cached chat members when participant lifecycle becomes inactive', () => {
    const membersKey = ['communications', 'telegram', 'chat-members', 'tgchat-1', 50, '', '']
    const existing = {
      pages: [
        {
          items: [
            {
              sender_id: 'user:42',
              sender_display_name: 'Former Member',
              message_count: 0,
              last_message_at: null,
              source: 'tdlib',
              provider_member_id: 'user:42',
              username: null,
              role: 'member',
              status: 'member',
              is_admin: false,
              is_owner: false,
              permissions: {},
              observed_at: null,
            },
          ],
          next_cursor: null,
        },
      ],
      pageParams: [null],
    }
    const setQueryData = vi.fn((queryKey, updater) =>
      typeof updater === 'function' ? updater(existing) : updater
    )
    const queryClient = {
      invalidateQueries: vi.fn(),
      getQueriesData: vi.fn().mockImplementation(({ queryKey }) => {
        if (JSON.stringify(queryKey) === JSON.stringify(['communications', 'telegram', 'chat-members'])) {
          return [[membersKey, existing]]
        }
        return []
      }),
      setQueryData
    }

    handleRealtimeEvent(
      {
        id: 'tg-participant-left-1',
        event: 'event',
        data: JSON.stringify({
          event: {
            event_type: 'telegram.participant.updated',
            payload: {
              telegram_chat_id: 'tgchat-1',
              participant: {
                sender_id: 'user:42',
                sender_display_name: 'Former Member',
                provider_member_id: 'user:42',
                source: 'tdlib',
                role: 'left',
                status: 'left',
                is_admin: false,
                is_owner: false,
                permissions: {
                  membership_state: 'left',
                },
                observed_at: '2026-06-17T00:00:00Z'
              }
            }
          }
        })
      },
      queryClient
    )

    const patched = setQueryData.mock.results[0]?.value
    expect(patched.pages[0].items).toEqual([])
  })
})
```

### `frontend/src/platform/bootstrap/realtimeTelegramProviderChatUpdates.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/platform/bootstrap/realtimeTelegramProviderChatUpdates.test.ts`
- Size bytes / Размер в байтах: `6862`
- Included characters / Включено символов: `6862`
- Truncated / Обрезано: `no`

```typescript
import { describe, expect, it, vi } from 'vitest'
import { handleRealtimeEvent } from './realtime'

function queryClientForChat(chat: Record<string, unknown>) {
  const chatsKey = ['communications', 'telegram', 'chats', 'account-1', 50]
  const chatDetailKey = ['communications', 'telegram', 'chat-detail', 'tgchat-1']
  const setQueryData = vi.fn((queryKey, updater) => {
    if (typeof updater !== 'function') return updater
    if (JSON.stringify(queryKey) === JSON.stringify(chatsKey)) return updater([chat])
    if (JSON.stringify(queryKey) === JSON.stringify(chatDetailKey)) return updater(chat)
    return updater(undefined)
  })
  return {
    invalidateQueries: vi.fn(),
    getQueriesData: vi.fn().mockImplementation(({ queryKey }) => {
      const key = JSON.stringify(queryKey)
      if (key === JSON.stringify(['communications', 'telegram', 'chats'])) return [[chatsKey, [chat]]]
      if (key === JSON.stringify(['communications', 'telegram', 'chat-detail'])) return [[chatDetailKey, chat]]
      return []
    }),
    setQueryData,
  }
}

describe('telegram provider chat.updated cache patching', () => {
  it('patches cached chat snapshots for provider notification settings updates', () => {
    const chat = {
      telegram_chat_id: 'tgchat-1',
      account_id: 'account-1',
      provider_chat_id: 'chat-1',
      chat_kind: 'private',
      title: 'Chat',
      username: null,
      sync_state: 'synced',
      last_message_at: null,
      metadata: { is_muted: false },
      created_at: '2026-06-16T09:00:00Z',
      updated_at: '2026-06-16T09:00:00Z',
    }
    const updatedChat = { ...chat, metadata: { ...chat.metadata, is_muted: true } }
    const queryClient = queryClientForChat(chat)

    handleRealtimeEvent(
      {
        id: 'tg-chat-updated-mute',
        event: 'event',
        data: JSON.stringify({
          event: {
            event_type: 'telegram.chat.updated',
            payload: {
              telegram_chat_id: 'tgchat-1',
              provider_chat_id: 'chat-1',
              action: 'provider_notification_settings_update',
              chat: updatedChat,
            },
          },
        }),
      },
      queryClient
    )

    expect(queryClient.setQueryData.mock.results[0]?.value[0].metadata.is_muted).toBe(true)
    expect(queryClient.setQueryData.mock.results[1]?.value.metadata.is_muted).toBe(true)
  })

  it('patches cached chat snapshots for provider chat position updates', () => {
    const chat = {
      telegram_chat_id: 'tgchat-1',
      account_id: 'account-1',
      provider_chat_id: 'chat-1',
      chat_kind: 'private',
      title: 'Chat',
      username: null,
      sync_state: 'synced',
      last_message_at: null,
      metadata: { is_archived: false, is_pinned: false },
      created_at: '2026-06-16T09:00:00Z',
      updated_at: '2026-06-16T09:00:00Z',
    }
    const updatedChat = {
      ...chat,
      metadata: { ...chat.metadata, is_archived: true, is_pinned: true, provider_folder_id: 7 },
    }
    const queryClient = queryClientForChat(chat)

    handleRealtimeEvent(
      {
        id: 'tg-chat-updated-position',
        event: 'event',
        data: JSON.stringify({
          event: {
            event_type: 'telegram.chat.updated',
            payload: {
              telegram_chat_id: 'tgchat-1',
              provider_chat_id: 'chat-1',
              action: 'provider_chat_position_update',
              chat: updatedChat,
            },
          },
        }),
      },
      queryClient
    )

    expect(queryClient.setQueryData.mock.results[0]?.value[0].metadata.is_archived).toBe(true)
    expect(queryClient.setQueryData.mock.results[0]?.value[0].metadata.is_pinned).toBe(true)
    expect(queryClient.setQueryData.mock.results[1]?.value.metadata.provider_folder_id).toBe(7)
  })

  it('patches cached chat snapshots for provider folder label updates', () => {
    const chat = {
      telegram_chat_id: 'tgchat-1',
      account_id: 'account-1',
      provider_chat_id: 'chat-1',
      chat_kind: 'private',
      title: 'Chat',
      username: null,
      sync_state: 'synced',
      last_message_at: null,
      metadata: { folder_labels: ['Unknown folder 7'], folder_name: 'Unknown folder 7' },
      created_at: '2026-06-16T09:00:00Z',
      updated_at: '2026-06-16T09:00:00Z',
    }
    const updatedChat = {
      ...chat,
      metadata: {
        ...chat.metadata,
        folder_labels: ['Projects'],
        folder_name: 'Projects',
        provider_folder_id: 7,
      },
    }
    const queryClient = queryClientForChat(chat)

    handleRealtimeEvent(
      {
        id: 'tg-chat-updated-folders',
        event: 'event',
        data: JSON.stringify({
          event: {
            event_type: 'telegram.chat.updated',
            payload: {
              telegram_chat_id: 'tgchat-1',
              provider_chat_id: 'chat-1',
              action: 'provider_chat_folder_labels_update',
              chat: updatedChat,
            },
          },
        }),
      },
      queryClient
    )

    expect(queryClient.setQueryData.mock.results[0]?.value[0].metadata.folder_labels).toEqual([
      'Projects',
    ])
    expect(queryClient.setQueryData.mock.results[1]?.value.metadata.folder_name).toBe('Projects')
    expect(queryClient.setQueryData.mock.results[1]?.value.metadata.provider_folder_id).toBe(7)
  })

  it('replaces cached chat snapshots when provider folder labels fall back to unknown', () => {
    const chat = {
      telegram_chat_id: 'tgchat-1',
      account_id: 'account-1',
      provider_chat_id: 'chat-1',
      chat_kind: 'private',
      title: 'Chat',
      username: null,
      sync_state: 'synced',
      last_message_at: null,
      metadata: { folder_labels: ['Projects'], folder_name: 'Projects', provider_folder_id: 7 },
      created_at: '2026-06-16T09:00:00Z',
      updated_at: '2026-06-16T09:00:00Z',
    }
    const updatedChat = {
      ...chat,
      metadata: {
        folder_labels: ['Unknown folder 7'],
        folder_name: 'Unknown folder 7',
        provider_folder_id: 7,
      },
    }
    const queryClient = queryClientForChat(chat)

    handleRealtimeEvent(
      {
        id: 'tg-chat-updated-folders-fallback',
        event: 'event',
        data: JSON.stringify({
          event: {
            event_type: 'telegram.chat.updated',
            payload: {
              telegram_chat_id: 'tgchat-1',
              provider_chat_id: 'chat-1',
              action: 'provider_chat_folder_labels_update',
              chat: updatedChat,
            },
          },
        }),
      },
      queryClient
    )

    expect(queryClient.setQueryData.mock.results[0]?.value[0].metadata.folder_labels).toEqual([
      'Unknown folder 7',
    ])
    expect(queryClient.setQueryData.mock.results[1]?.value.metadata.folder_name).toBe(
      'Unknown folder 7'
    )
  })
})
```

### `frontend/src/platform/bootstrap/realtimeTelegramTopicPatches.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/platform/bootstrap/realtimeTelegramTopicPatches.test.ts`
- Size bytes / Размер в байтах: `3459`
- Included characters / Включено символов: `3459`
- Truncated / Обрезано: `no`

```typescript
import { describe, expect, it, vi } from 'vitest'
import { handleRealtimeEvent } from './realtime'

describe('telegram realtime topic cache patch handling', () => {
  it('patches cached telegram topic lists for topic update events', () => {
    const topicsKey = ['communications', 'telegram', 'topics', 'telegram-chat-1', 100]
    const topicSearchKey = ['communications', 'telegram', 'topic-search', 'telegram-chat-1', 'release', 50]
    const existingTopic = {
      topic_id: 'telegram-topic-old',
      telegram_chat_id: 'telegram-chat-1',
      account_id: 'account-1',
      provider_topic_id: 7,
      provider_chat_id: '-100123',
      title: 'Older topic',
      icon_emoji: null,
      is_pinned: false,
      is_closed: false,
      unread_count: 0,
      last_message_at: null,
      metadata: {},
      created_at: '2026-06-16T09:00:00Z',
      updated_at: '2026-06-16T09:00:00Z'
    }
    const updatedTopic = {
      topic_id: 'telegram-topic-42',
      telegram_chat_id: 'telegram-chat-1',
      account_id: 'account-1',
      provider_topic_id: 42,
      provider_chat_id: '-100123',
      title: 'Release notes',
      icon_emoji: '5368324170671202286',
      is_pinned: true,
      is_closed: false,
      unread_count: 0,
      last_message_at: null,
      metadata: {},
      created_at: '2026-06-16T09:00:00Z',
      updated_at: '2026-06-17T09:00:00Z'
    }
    const topicsResponse = { telegram_chat_id: 'telegram-chat-1', items: [existingTopic] }
    const searchResponse = { telegram_chat_id: 'telegram-chat-1', items: [] }
    const setQueryData = vi.fn((queryKey, updater) => {
      if (typeof updater !== 'function') return updater
      if (JSON.stringify(queryKey) === JSON.stringify(topicsKey)) return updater(topicsResponse)
      if (JSON.stringify(queryKey) === JSON.stringify(topicSearchKey)) return updater(searchResponse)
      return updater(undefined)
    })
    const queryClient = {
      invalidateQueries: vi.fn(),
      getQueriesData: vi.fn().mockImplementation(({ queryKey }) => {
        if (JSON.stringify(queryKey) === JSON.stringify(['communications', 'telegram'])) {
          return [[topicsKey, topicsResponse], [topicSearchKey, searchResponse]]
        }
        return []
      }),
      setQueryData
    }

    handleRealtimeEvent(
      {
        id: 'tg-topic-1',
        event: 'event',
        data: JSON.stringify({
          event: {
            event_type: 'telegram.topic.updated',
            subject: { id: 'telegram-topic-42', kind: 'telegram_topic' },
            payload: {
              account_id: 'account-1',
              telegram_chat_id: 'telegram-chat-1',
              provider_chat_id: '-100123',
              provider_topic_id: 42,
              topic_id: 'telegram-topic-42',
              topic: updatedTopic
            }
          }
        })
      },
      queryClient
    )

    const patchedTopics = setQueryData.mock.results[0]?.value
    expect(patchedTopics.items[0]).toMatchObject({
      topic_id: 'telegram-topic-42',
      title: 'Release notes',
      is_pinned: true
    })

    const patchedSearch = setQueryData.mock.results[1]?.value
    expect(patchedSearch.items[0].topic_id).toBe('telegram-topic-42')
    expect(queryClient.invalidateQueries).toHaveBeenCalledWith({ queryKey: ['communications', 'telegram', 'topics'] })
    expect(queryClient.invalidateQueries).toHaveBeenCalledWith({ queryKey: ['communications', 'telegram', 'topic-search'] })
  })
})
```

### `frontend/src/platform/bootstrap/realtimeWhatsAppCachePatches.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/platform/bootstrap/realtimeWhatsAppCachePatches.test.ts`
- Size bytes / Размер в байтах: `22551`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```typescript
import { describe, expect, it, vi } from 'vitest'
import { handleRealtimeEvent } from './realtime'
import type { RealtimeQueryClient } from './realtime'

describe('whatsapp realtime cache patch handling', () => {
	it('patches cached whatsapp conversation rows for dialog lifecycle events', () => {
		const conversationsKey = ['communications', 'whatsapp', 'conversations', 'account-1', 50]
		const conversationDetailKey = ['communications', 'whatsapp', 'conversation-detail', 'wa-chat-1']
		const conversations = [
			{
				conversation_id: 'wa-chat-1',
				account_id: 'account-1',
				provider_chat_id: 'wa-chat-1',
				chat_kind: 'group',
				title: 'Family',
				last_message_at: '2026-06-16T09:00:00Z',
				metadata: {
					is_pinned: false,
					is_archived: false,
					is_muted: false,
					is_unread: false,
					unread_count: 0,
				},
				created_at: '2026-06-16T09:00:00Z',
				updated_at: '2026-06-16T09:00:00Z',
			},
		]
		const detail = conversations[0]
		const setQueryData = vi.fn((queryKey, updater) =>
			typeof updater === 'function'
				? String(queryKey[2]) === 'conversation-detail'
					? updater(detail)
					: updater(conversations)
				: updater
		)
		const queryClient = {
			invalidateQueries: vi.fn(),
			getQueriesData: vi.fn(({ queryKey }) =>
				String(queryKey[2]) === 'conversation-detail'
					? [[conversationDetailKey, detail]]
					: [[conversationsKey, conversations]]
			),
			setQueryData,
		} as unknown as RealtimeQueryClient

		handleRealtimeEvent(
			{
				id: 'wa-dialog-1',
				event: 'event',
				data: JSON.stringify({
					event: {
						event_type: 'whatsapp.dialog.updated',
						payload: {
							account_id: 'account-1',
							conversation_id: 'wa-chat-1',
							provider_chat_id: 'wa-chat-1',
							chat_title: 'Family',
							chat_kind: 'group',
							is_pinned: true,
							is_archived: true,
							is_muted: true,
							is_unread: true,
							unread_count: 4,
							participant_count: 7,
							observed_at: '2026-06-16T09:02:00Z',
						},
					},
				}),
			},
			queryClient
		)

		const patchedConversationList = setQueryData.mock.results[0]?.value
		const patchedConversationDetail = setQueryData.mock.results[1]?.value
		expect(patchedConversationList[0].metadata).toMatchObject({
			is_pinned: true,
			is_archived: true,
			is_muted: true,
			is_unread: true,
			unread_count: 4,
			participant_count: 7,
		})
		expect(patchedConversationDetail.metadata).toMatchObject({
			is_pinned: true,
			is_archived: true,
			is_muted: true,
			is_unread: true,
			unread_count: 4,
			participant_count: 7,
		})
	})

	it('patches cached whatsapp message reaction summary for reaction events', () => {
		const messageKey = ['communications', 'whatsapp', 'messages', 'account-1', 'chat-1', 50]
		const messages = [
			{
				message_id: 'wa-msg-1',
				raw_record_id: 'raw-1',
				account_id: 'account-1',
				provider_message_id: 'provider-1',
				provider_chat_id: 'chat-1',
				chat_title: 'Chat',
				sender: 'sender-1',
				sender_display_name: 'Sender',
				text: 'Hello',
				occurred_at: '2026-06-16T09:00:00Z',
				projected_at: '2026-06-16T09:00:01Z',
				channel_kind: 'whatsapp_web' as const,
				delivery_state: 'received',
				metadata: {},
			},
		]
		const setQueryData = vi.fn((queryKey, updater) =>
			typeof updater === 'function' ? updater(messages) : updater
		)
		const queryClient = {
			invalidateQueries: vi.fn(),
			getQueriesData: vi.fn().mockReturnValue([[messageKey, messages]]),
			setQueryData,
		}

		handleRealtimeEvent(
			{
				id: 'wa-1',
				event: 'event',
				data: JSON.stringify({
					event: {
						event_type: 'whatsapp.reaction.changed',
						payload: {
							account_id: 'account-1',
							provider_chat_id: 'chat-1',
							message_id: 'wa-msg-1',
							reaction: '+1',
							is_active: true,
						},
					},
				}),
			},
			queryClient
		)

		const patchedMessages = setQueryData.mock.results[0]?.value
		expect(patchedMessages[0].metadata.reaction_summary.reactions[0]).toMatchObject({
			reaction: '+1',
			count: 1,
		})
	})

	it('patches cached whatsapp session link-state and runtime metadata events', () => {
		const sessionsKey = ['integrations', 'whatsapp', 'sessions', 'account-1', 50]
		const runtimeStatusKey = ['integrations', 'whatsapp', 'runtime', 'status', 'account-1']
		const sessions = [
			{
				session_id: 'session-1',
				account_id: 'account-1',
				device_name: 'Макошь Desktop',
				companion_runtime: 'fixture' as const,
				link_state: 'qr_pending' as const,
				local_state_path: 'docker/data/whatsapp/session-1',
				last_sync_at: null,
				metadata: {},
				created_at: '2026-06-16T09:00:00Z',
				updated_at: '2026-06-16T09:00:00Z',
			},
		]
		const runtimeStatus = {
			account_id: 'account-1',
			provider_kind: 'whatsapp_web',
			provider_shape: 'whatsapp_web_companion',
			runtime_kind: 'fixture',
			status: 'qr_pending',
			fixture_runtime: true,
			live_runtime_available: false,
			live_send_available: false,
			qr_pairing_available: true,
			pair_code_available: true,
			media_download_available: false,
			media_upload_available: false,
			session_restore_available: false,
			session_secret_ref: null,
			runtime_blockers: ['whatsapp_session_link_required'],
			last_error: null,
			updated_at: '2026-06-16T09:00:00Z',
		}
		const setQueryData = vi.fn((queryKey, updater) =>
			typeof updater === 'function'
				? String(queryKey[3]) === 'status'
					? updater(runtimeStatus)
					: updater(sessions)
				: updater
		)
		const queryClient = {
			invalidateQueries: vi.fn(),
			getQueriesData: vi.fn(({ queryKey }) =>
				String(queryKey[3]) === 'status'
					? [[runtimeStatusKey, runtimeStatus]]
					: [[sessionsKey, sessions]]
			),
			setQueryData,
		} as unknown as RealtimeQueryClient

		handleRealtimeEvent(
			{
				id: 'wa-2',
				event: 'event',
				data: JSON.stringify({
					event: {
						event_type: 'whatsapp.session.link_state_changed',
						payload: {
							account_id: 'account-1',
							link_state: 'linked',
							occurred_at: '2026-06-16T09:01:00Z',
						},
					},
				}),
			},
			queryClient
		)
		handleRealtimeEvent(
			{
				id: 'wa-3',
				event: 'event',
				data: JSON.stringify({
					event: {
						event_type: 'whatsapp.runtime.status_changed',
						payload: {
							account_id: 'account-1',
							status: 'available',
							source: 'runtime_start',
							occurred_at: '2026-06-16T09:02:00Z',
						},
					},
				}),
			},
			queryClient
		)

		expect(setQueryData.mock.results[0]?.value[0].link_state).toBe('linked')
		expect(setQueryData.mock.results[1]?.value.status).toBe('linked')
		expect(setQueryData.mock.results[2]?.value[0].metadata.runtime_status).toBe('available')
		expect(setQueryData.mock.results[2]?.value[0].metadata.runtime_status_source).toBe(
			'runtime_start'
		)
		expect(setQueryData.mock.results[3]?.value.status).toBe('available')
		expect(setQueryData.mock.results[3]?.value.runtime_blockers).toEqual([
			'whatsapp_session_link_required',
		])
	})

	it('patches cached whatsapp provider commands for command status events', () => {
		const commandsKey = ['integrations', 'whatsapp', 'commands', 'account-1', 25]
		const commands = [
			{
				command_id: 'wa-cmd-1',
				account_id: 'account-1',
				command_kind: 'publish_status',
				idempotency_key: 'status:1',
				provider_chat_id: 'status-feed',
				provider_message_id: null,
				capability_state: 'blocked',
				action_class: 'provider_write',
				confirmation_decision: 'not_required',
				status: 'queued',
				retry_count: 0,
				max_retries: 3,
				last_error: null,
				result_payload: {},
				audit_metadata: {},
				provider_state: {},
				reconciliation_status: 'not_observed',
				next_attempt_at: null,
				last_attempt_at: null,
				provider_observed_at: null,
				reconciled_at: null,
				dead_lettered_at: null,
				completed_at: null,
				created_at: '2026-06-16T09:00:00Z',
				updated_at: '2026-06-16T09:00:00Z',
			},
		]
		const setQueryData = vi.fn((queryKey, updater) =>
			typeof updater === 'function' ? updater(commands) : updater
		)
		const queryClient = {
			invalidateQueries: vi.fn(),
			getQueriesData: vi.fn(({ queryKey }) =>
				String(queryKey[2]) === 'commands' ? [[commandsKey, commands]] : []
			),
			setQueryData,
		} as unknown as RealtimeQueryClient

		handleRealtimeEvent(
			{
				id: 'wa-cmd-status-1',
				event: 'event',
				data: JSON.stringify({
					event: {
						event_type: 'whatsapp.command.status_changed',
						payload: {
							account_id: 'account-1',
							command_id: 'wa-cmd-1',
							command_kind: 'publish_status',
							provider_chat_id: 'status-feed',
							status: 'completed',
							reconciliation_status: 'observed',
							provider_observed_at: '2026-06-16T09:01:00Z',
							completed_at: '2026-06-16T09:01:00Z',
						},
					},
				}),
			},
			queryClient
		)

		expect(setQueryData).toHaveBeenCalledTimes(1)
		expect(setQueryData.mock.results[0]?.value[0]).toMatchObject({
			command_id: 'wa-cmd-1',
			status: 'completed',
			reconciliation_status: 'observed',
			provider_observed_at: '2026-06-16T09:01:00Z',
			completed_at: '2026-06-16T09:01:00Z',
		})
	})

	it('patches cached whatsapp presence and call sync snapshots from direct runtime events', () => {
		const presenceKey = ['integrations', 'whatsapp', 'runtime', 'sync-presence', 'account-1', 'chat-1', 8]
		const callsKey = ['integrations', 'whatsapp', 'runtime', 'sync-calls', 'account-1', 'chat-1', 8]
		let presenceItems = [
			{
				identity_id: 'identity-1',
				account_id: 'account-1',
				channel_kind: 'whatsapp_web',
				provider_chat_id: 'chat-1',
				provider_identity_id: 'wa:+111',
				identity_kind: 'whatsapp',
				display_name: 'Alice',
				address: '+111',
				presence_state: 'offline',
				last_seen_at: '2026-06-16T09:00:00Z',
				observed_at: '2026-06-16T09:00:00Z',
				identity_metadata: {},
			},
		]
		let callItems = [
			{
				call_id: 'call-1',
				account_id: 'account-1',
				provider_call_id: 'provider-call-1',
				provider_chat_id: 'chat-1',
				direction: 'incoming',
				call_state: 'ringing',
				started_at: '2026-06-16T09:00:00Z',
				ended_at: null,
				observed_at: '2026-06-16T09:00:00Z',
				metadata: {},
			},
		]
		const setQueryData = vi.fn((queryKey, updater) => {
			if (typeof updater !== 'function') return updater
			if (String(queryKey[3]) === 'sync-presence') {
				presenceItems = updater(presenceItems)
				return presenceItems
			}
			if (String(queryKey[3]) === 'sync-calls') {
				callItems = updater(callItems)
				return callItems
			}
			return undefined
		})
		const queryClient = {
			invalidateQueries: vi.fn(),
			getQueriesData: vi.fn(({ queryKey }) => {
				if (String(queryKey[3]) === 'sync-presence') return [[presenceKey, presenceItems]]
				if (String(queryKey[3]) === 'sync-calls') return [[callsKey, callItems]]
				return []
			}),
			setQueryData,
		} as unknown as RealtimeQueryClient

		handleRealtimeEvent(
			{
				id: 'wa-presence-patch-1',
				event: 'event',
				data: JSON.stringify({
					event: {
						event_type: 'whatsapp.presence.changed',
						payload: {
							account_id: 'account-1',
							identity_id: 'identity-1',
							provider_chat_id: 'chat-1',
							provider_identity_id: 'wa:+111',
							identity_kind: 'whatsapp',
							display_name: 'Alice Cooper',
							address: '+111',
							presence_state: 'typing',
							last_seen_at: '2026-06-16T09:01:00Z',
							observed_at: '2026-06-16T09:01:00Z',
						},
					},
				}),
			},
			queryClient
		)
		handleRealtimeEvent(
			{
				id: 'wa-call-patch-1',
				event: 'event',
				data: JSON.stringify({
					event: {
						event_type: 'whatsapp.call.updated',
						payload: {
							account_id: 'account-1',
							call_id: 'call-1',
							provider_call_id: 'provider-call-1',
							provider_chat_id: 'chat-1',
							direction: 'incoming',
							call_state: 'connected',
							started_at: '2026-06-16T09:00:00Z',
							ended_at: '2026-06-16T09:05:00Z',
							observed_at: '2026-06-16T09:05:00Z',

```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `frontend/src/platform/bootstrap/realtimeWhatsAppInvalidation.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/platform/bootstrap/realtimeWhatsAppInvalidation.test.ts`
- Size bytes / Размер в байтах: `6840`
- Included characters / Включено символов: `6840`
- Truncated / Обрезано: `no`

```typescript
import { describe, expect, it, vi } from 'vitest'
import { handleRealtimeEvent } from './realtime'

describe('whatsapp realtime invalidation handling', () => {
	it('invalidates whatsapp message queries for whatsapp message events', () => {
		const queryClient = { invalidateQueries: vi.fn() }

		handleRealtimeEvent(
			{
				id: 'wa-10',
				event: 'event',
				data: JSON.stringify({
					event: {
						event_type: 'whatsapp.message.created',
					},
				}),
			},
			queryClient
		)

		expect(queryClient.invalidateQueries).toHaveBeenCalledTimes(1)
		expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
			queryKey: ['communications', 'whatsapp', 'messages'],
		})
	})

	it('invalidates whatsapp conversation and message queries for dialog events', () => {
		const queryClient = { invalidateQueries: vi.fn() }

		handleRealtimeEvent(
			{
				id: 'wa-10b',
				event: 'event',
				data: JSON.stringify({
					event: {
						event_type: 'whatsapp.dialog.updated',
					},
				}),
			},
			queryClient
		)

		expect(queryClient.invalidateQueries).toHaveBeenCalledTimes(3)
		expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
			queryKey: ['communications', 'whatsapp', 'conversations'],
		})
		expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
			queryKey: ['communications', 'whatsapp', 'conversation-detail'],
		})
		expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
			queryKey: ['communications', 'whatsapp', 'messages'],
		})
	})

	it('invalidates whatsapp session and capability queries for runtime lifecycle events', () => {
		const queryClient = { invalidateQueries: vi.fn() }

		handleRealtimeEvent(
			{
				id: 'wa-11',
				event: 'event',
				data: JSON.stringify({
					event: {
						event_type: 'whatsapp.runtime.status_changed',
					},
				}),
			},
			queryClient
		)

		expect(queryClient.invalidateQueries).toHaveBeenCalledTimes(5)
		expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
			queryKey: ['integrations', 'whatsapp', 'sessions'],
		})
		expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
			queryKey: ['integrations', 'whatsapp', 'capabilities'],
		})
		expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
			queryKey: ['integrations', 'whatsapp', 'account-capabilities'],
		})
		expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
			queryKey: ['integrations', 'whatsapp', 'runtime', 'status'],
		})
		expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
			queryKey: ['integrations', 'whatsapp', 'runtime', 'health'],
		})
	})

	it('invalidates whatsapp session and capability queries for media lifecycle events', () => {
		const queryClient = { invalidateQueries: vi.fn() }

		handleRealtimeEvent(
			{
				id: 'wa-12',
				event: 'event',
				data: JSON.stringify({
					event: {
						event_type: 'whatsapp.media.download.progress',
					},
				}),
			},
			queryClient
		)

		expect(queryClient.invalidateQueries).toHaveBeenCalledTimes(7)
		expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
			queryKey: ['integrations', 'whatsapp', 'commands'],
		})
		expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
			queryKey: ['integrations', 'whatsapp', 'sessions'],
		})
		expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
			queryKey: ['integrations', 'whatsapp', 'capabilities'],
		})
		expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
			queryKey: ['integrations', 'whatsapp', 'account-capabilities'],
		})
		expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
			queryKey: ['integrations', 'whatsapp', 'runtime', 'status'],
		})
		expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
			queryKey: ['integrations', 'whatsapp', 'runtime', 'health'],
		})
		expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
			queryKey: ['integrations', 'whatsapp', 'runtime', 'sync-media'],
		})
	})

	it('invalidates whatsapp conversation queries for participant events', () => {
		const queryClient = { invalidateQueries: vi.fn() }

		handleRealtimeEvent(
			{
				id: 'wa-13',
				event: 'event',
				data: JSON.stringify({
					event: {
						event_type: 'whatsapp.participant.changed',
					},
				}),
			},
			queryClient
		)

		expect(queryClient.invalidateQueries).toHaveBeenCalledTimes(4)
		expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
			queryKey: ['communications', 'whatsapp', 'conversations'],
		})
		expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
			queryKey: ['communications', 'whatsapp', 'conversation-detail'],
		})
		expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
			queryKey: ['communications', 'whatsapp', 'chat-members'],
		})
		expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
			queryKey: ['integrations', 'whatsapp', 'runtime', 'sync-contacts'],
		})
	})

	it('invalidates whatsapp runtime sync queries for presence, calls and statuses', () => {
		const queryClient = { invalidateQueries: vi.fn() }

		handleRealtimeEvent(
			{
				id: 'wa-presence-1',
				event: 'event',
				data: JSON.stringify({
					event: {
						event_type: 'whatsapp.presence.changed',
					},
				}),
			},
			queryClient
		)
		handleRealtimeEvent(
			{
				id: 'wa-call-1',
				event: 'event',
				data: JSON.stringify({
					event: {
						event_type: 'whatsapp.call.updated',
					},
				}),
			},
			queryClient
		)
		handleRealtimeEvent(
			{
				id: 'wa-status-1',
				event: 'event',
				data: JSON.stringify({
					event: {
						event_type: 'whatsapp.status.updated',
					},
				}),
			},
			queryClient
		)

		expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
			queryKey: ['integrations', 'whatsapp', 'runtime', 'sync-presence'],
		})
		expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
			queryKey: ['integrations', 'whatsapp', 'runtime', 'sync-calls'],
		})
		expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
			queryKey: ['integrations', 'whatsapp', 'runtime', 'sync-statuses'],
		})
	})

	it('invalidates whatsapp runtime sync queries for chats, history and members on sync lifecycle events', () => {
		const queryClient = { invalidateQueries: vi.fn() }

		handleRealtimeEvent(
			{
				id: 'wa-sync-1',
				event: 'event',
				data: JSON.stringify({
					event: {
						event_type: 'whatsapp.sync.completed',
						payload: {
							scope: 'history',
							provider_chat_id: 'chat-1',
						},
					},
				}),
			},
			queryClient
		)

		expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
			queryKey: ['integrations', 'whatsapp', 'runtime', 'sync-chats'],
		})
		expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
			queryKey: ['integrations', 'whatsapp', 'runtime', 'sync-history'],
		})
		expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
			queryKey: ['integrations', 'whatsapp', 'runtime', 'sync-members'],
		})
	})
})
```

### `frontend/src/platform/bootstrap/realtimeZoomInvalidation.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/platform/bootstrap/realtimeZoomInvalidation.test.ts`
- Size bytes / Размер в байтах: `1686`
- Included characters / Включено символов: `1686`
- Truncated / Обрезано: `no`

```typescript
import { describe, expect, it, vi } from 'vitest'
import { handleRealtimeEvent } from './realtime'

describe('zoom realtime invalidation handling', () => {
  it('invalidates zoom integration keys and current call caches for zoom events', () => {
    const queryClient = { invalidateQueries: vi.fn() }

    handleRealtimeEvent(
      {
        id: 'zoom-1',
        event: 'event',
        data: JSON.stringify({
          event: {
            event_type: 'zoom.transcript.observed',
          },
        }),
      },
      queryClient
    )

    expect(queryClient.invalidateQueries).toHaveBeenCalledTimes(8)
    expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
      queryKey: ['integrations', 'zoom', 'accounts'],
    })
    expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
      queryKey: ['integrations', 'zoom', 'capabilities'],
    })
    expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
      queryKey: ['integrations', 'zoom', 'runtime', 'status'],
    })
    expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
      queryKey: ['integrations', 'zoom', 'webhook-subscriptions'],
    })
    expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
      queryKey: ['integrations', 'zoom', 'provider-calls'],
    })
    expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
      queryKey: ['integrations', 'zoom', 'provider-call-transcript'],
    })
    expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
      queryKey: ['integrations', 'zoom', 'recording-imports'],
    })
    expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
      queryKey: ['integrations', 'zoom', 'audit-events'],
    })
  })
})
```

### `frontend/src/platform/bootstrap/useTelegramMediaUploadWorkflow.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/platform/bootstrap/useTelegramMediaUploadWorkflow.test.ts`
- Size bytes / Размер в байтах: `2177`
- Included characters / Включено символов: `2177`
- Truncated / Обрезано: `no`

```typescript
import { describe, expect, it, vi } from 'vitest'
import {
  primeTelegramUploadCommandQueues,
  telegramMediaTypeForFile,
} from './useTelegramMediaUploadWorkflow'

describe('telegramMediaTypeForFile', () => {
  it('maps common desktop files to supported Telegram upload kinds', () => {
    expect(telegramMediaTypeForFile({ name: 'photo.jpg', type: 'image/jpeg' } as File)).toBe('photo')
    expect(telegramMediaTypeForFile({ name: 'clip.mp4', type: 'video/mp4' } as File)).toBe('video')
    expect(telegramMediaTypeForFile({ name: 'voice.ogg', type: 'audio/ogg' } as File)).toBe('audio')
    expect(telegramMediaTypeForFile({ name: 'fun.gif', type: 'image/gif' } as File)).toBe('animation')
    expect(telegramMediaTypeForFile({ name: 'archive.zip', type: 'application/zip' } as File)).toBe('document')
  })

  it('primes current command queues with a synthetic send_media row after upload success', () => {
    const commandsKey = ['integrations', 'telegram', 'commands', 'account-1', 20]
    const commands: Array<Record<string, unknown>> = []
    const setQueryData = vi.fn()
    const queryClient = {
      getQueriesData: vi.fn().mockReturnValue([[commandsKey, commands]]),
      setQueryData,
    }

    primeTelegramUploadCommandQueues(
      queryClient,
      {
        command_id: 'cmd-upload-1',
        account_id: 'account-1',
        provider_chat_id: 'chat-1',
        attachment_id: 'att-1',
        blob_id: 'blob-1',
        media_type: 'document',
        status: 'queued',
        reconciliation_status: 'not_observed',
      },
      'upload-note.txt',
      'hello'
    )

    expect(setQueryData).toHaveBeenCalledOnce()
    expect(setQueryData.mock.calls[0][0]).toEqual(commandsKey)
    expect(setQueryData.mock.calls[0][1][0]).toMatchObject({
      command_id: 'cmd-upload-1',
      account_id: 'account-1',
      command_kind: 'send_media',
      provider_chat_id: 'chat-1',
      status: 'queued',
      reconciliation_status: 'not_observed',
    })
    expect(setQueryData.mock.calls[0][1][0].payload).toMatchObject({
      attachment_id: 'att-1',
      blob_id: 'blob-1',
      filename: 'upload-note.txt',
      caption: 'hello',
    })
  })
})
```

### `frontend/src/platform/bootstrap/useTelegramMediaUploadWorkflow.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/platform/bootstrap/useTelegramMediaUploadWorkflow.ts`
- Size bytes / Размер в байтах: `5163`
- Included characters / Включено символов: `5163`
- Truncated / Обрезано: `no`

```typescript
import { useMutation, useQueryClient } from '@tanstack/vue-query'
import { z } from 'zod'
import { importCommunicationAttachment } from '../../domains/communications/api/attachmentImportApi'
import {
  uploadTelegramMedia,
  type TelegramMediaUploadKind,
} from '../../integrations/telegram/api/telegramMediaUpload'
import { patchTelegramCommandList } from '../../integrations/telegram/queries/realtimeTelegramCommandPatches'
import { telegramQueryKeys } from '../../integrations/telegram/queries/useTelegramQuery'
import type { TelegramProviderWriteCommand } from '../../integrations/telegram/types/telegram'

export type TelegramMediaUploadInput = {
  accountId: string
  providerChatId: string
  file: File
  caption?: string
}

const uploadInputSchema = z.object({
  accountId: z.string().trim().min(1),
  providerChatId: z.string().trim().min(1),
  file: z.custom<File>(
    (value) =>
      typeof value === 'object' &&
      value !== null &&
      typeof (value as File).arrayBuffer === 'function' &&
      typeof (value as File).size === 'number' &&
      (value as File).size > 0,
    'file is required'
  ),
  caption: z.string().trim().min(1).optional(),
})

export function useTelegramMediaUploadMutation() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: uploadTelegramMediaFile,
    onSuccess: (result, variables) => {
      primeTelegramUploadCommandQueues(queryClient, result, variables.file.name || undefined, variables.caption)
      queryClient.invalidateQueries({ queryKey: telegramQueryKeys.runtime })
      queryClient.invalidateQueries({ queryKey: ['integrations', 'telegram', 'commands', result.account_id] })
      queryClient.invalidateQueries({ queryKey: ['integrations', 'telegram', 'commands'] })
    },
  })
}

export function primeTelegramUploadCommandQueues(
  queryClient: {
    getQueriesData: <TData>(filters: { queryKey: readonly unknown[] }) => Array<
      [readonly unknown[], TData | undefined]
    >
    setQueryData: <TData>(queryKey: readonly unknown[], updater: TData) => unknown
  },
  result: Awaited<ReturnType<typeof uploadTelegramMedia>>,
  filename?: string,
  caption?: string
): void {
  for (const [queryKey, data] of queryClient.getQueriesData<TelegramProviderWriteCommand[]>({
    queryKey: ['integrations', 'telegram', 'commands'],
  })) {
    const updated = patchTelegramCommandList(
      queryKey,
      data,
      'telegram.media.upload.started',
      {
        command_id: result.command_id,
        account_id: result.account_id,
        provider_chat_id: result.provider_chat_id,
        command_kind: 'send_media',
        idempotency_key: result.command_id,
        capability_state: 'available',
        action_class: 'provider_write',
        confirmation_decision: 'confirmed',
        status: result.status,
        retry_count: 0,
        max_retries: 3,
        reconciliation_status: result.reconciliation_status,
        payload: {
          attachment_id: result.attachment_id,
          blob_id: result.blob_id,
          media_type: result.media_type,
          filename: filename?.trim() || undefined,
          caption: caption?.trim() || undefined,
        },
        target_ref: {
          provider_chat_id: result.provider_chat_id,
          attachment_id: result.attachment_id,
          blob_id: result.blob_id,
        },
      }
    )
    if (updated !== data) {
      queryClient.setQueryData(queryKey, updated)
    }
  }
}

export async function uploadTelegramMediaFile(input: TelegramMediaUploadInput) {
  const parsed = uploadInputSchema.parse(input)
  const contentBase64 = await fileToBase64(parsed.file)
  const imported = await importCommunicationAttachment({
    account_id: parsed.accountId.trim(),
    channel_kind: 'telegram',
    filename: parsed.file.name || undefined,
    content_type: parsed.file.type || 'application/octet-stream',
    content_base64: contentBase64,
    source_kind: 'telegram_composer',
    metadata: {
      composer: 'telegram',
      last_modified: parsed.file.lastModified || undefined,
    },
  })

  return uploadTelegramMedia({
    account_id: parsed.accountId.trim(),
    provider_chat_id: parsed.providerChatId.trim(),
    attachment_id: imported.attachment_id,
    blob_id: imported.blob_id,
    media_type: telegramMediaTypeForFile(parsed.file),
    caption: parsed.caption?.trim(),
    filename: parsed.file.name || undefined,
  })
}

export function telegramMediaTypeForFile(file: Pick<File, 'name' | 'type'>): TelegramMediaUploadKind {
  const type = file.type.toLowerCase()
  const name = file.name.toLowerCase()
  if (type === 'image/gif' || name.endsWith('.gif')) return 'animation'
  if (type.startsWith('image/')) return 'photo'
  if (type.startsWith('video/')) return 'video'
  if (type.startsWith('audio/')) return 'audio'
  return 'document'
}

async function fileToBase64(file: File): Promise<string> {
  const bytes = new Uint8Array(await file.arrayBuffer())
  let binary = ''
  const chunkSize = 0x8000
  for (let offset = 0; offset < bytes.length; offset += chunkSize) {
    binary += String.fromCharCode(...bytes.subarray(offset, offset + chunkSize))
  }
  return btoa(binary)
}
```

### `frontend/src/platform/config/env.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/platform/config/env.test.ts`
- Size bytes / Размер в байтах: `1387`
- Included characters / Включено символов: `1387`
- Truncated / Обрезано: `no`

```typescript
import { describe, expect, it } from 'vitest'
import { loadFrontendConfig } from './env'

describe('frontend env config', () => {
	it('uses Макошь env names and default backend URL', () => {
		const config = loadFrontendConfig({
			VITE_MAKOSH_LOCAL_API_SECRET: 'dev-secret'
		})

		expect(config.apiBaseUrl).toBe('http://127.0.0.1:8080')
		expect(config.apiSecret).toBe('dev-secret')
		expect(config.sseUrl).toBe('http://127.0.0.1:8080/api/events/stream')
		expect(config.webSocketUrl).toBe('ws://127.0.0.1:8080/api/events/ws')
		expect(config.realtimeTransport).toBe('sse')
	})

	it('rejects missing local API secret', () => {
		expect(() => loadFrontendConfig({})).toThrow('VITE_MAKOSH_LOCAL_API_SECRET is required')
	})

	it('accepts explicit Макошь backend URL', () => {
		const config = loadFrontendConfig({
			VITE_MAKOSH_API_BASE_URL: 'http://127.0.0.1:9090/',
			VITE_MAKOSH_LOCAL_API_SECRET: 'dev-secret'
		})

		expect(config.apiBaseUrl).toBe('http://127.0.0.1:9090')
		expect(config.sseUrl).toBe('http://127.0.0.1:9090/api/events/stream')
		expect(config.webSocketUrl).toBe('ws://127.0.0.1:9090/api/events/ws')
	})

	it('can opt back to WebSocket transport selection', () => {
		const config = loadFrontendConfig({
			VITE_MAKOSH_LOCAL_API_SECRET: 'dev-secret',
			VITE_MAKOSH_REALTIME_TRANSPORT: 'websocket'
		})

		expect(config.realtimeTransport).toBe('websocket')
	})
})
```

### `frontend/src/platform/config/env.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/platform/config/env.ts`
- Size bytes / Размер в байтах: `1767`
- Included characters / Включено символов: `1767`
- Truncated / Обрезано: `no`

```typescript
export type FrontendConfig = {
	apiBaseUrl: string
	apiSecret: string
	sseUrl: string
	webSocketUrl: string
	realtimeTransport: RealtimeTransportPreference
}

export type RealtimeTransportPreference = 'websocket' | 'sse'
type EnvSource = Record<string, string | boolean | undefined>

const DEFAULT_API_BASE_URL = 'http://127.0.0.1:8080'

export function loadFrontendConfig(env: EnvSource = import.meta.env): FrontendConfig {
	const apiBaseUrl = normalizeBaseUrl(
		stringValue(env.VITE_MAKOSH_API_BASE_URL) ?? DEFAULT_API_BASE_URL
	)
	const apiSecret = stringValue(env.VITE_MAKOSH_LOCAL_API_SECRET)

	if (!apiSecret) {
		throw new Error('VITE_MAKOSH_LOCAL_API_SECRET is required')
	}

	return {
		apiBaseUrl,
		apiSecret,
		sseUrl: stringValue(env.VITE_MAKOSH_SSE_URL) ?? `${apiBaseUrl}/api/events/stream`,
		webSocketUrl:
			stringValue(env.VITE_MAKOSH_WEBSOCKET_URL) ?? defaultWebSocketUrl(apiBaseUrl),
		realtimeTransport: realtimeTransportPreference(env.VITE_MAKOSH_REALTIME_TRANSPORT)
	}
}

function stringValue(value: string | boolean | undefined): string | undefined {
	return typeof value === 'string' && value.trim().length > 0 ? value.trim() : undefined
}

function normalizeBaseUrl(value: string): string {
	return value.replace(/\/+$/, '')
}

function defaultWebSocketUrl(apiBaseUrl: string): string {
	const parsed = new URL(apiBaseUrl)
	parsed.protocol = parsed.protocol === 'https:' ? 'wss:' : 'ws:'
	parsed.pathname = '/api/events/ws'
	parsed.search = ''
	parsed.hash = ''
	return parsed.toString().replace(/\/$/, '')
}

function realtimeTransportPreference(
	value: string | boolean | undefined
): RealtimeTransportPreference {
	const normalized = stringValue(value)?.toLowerCase()
	if (normalized === 'websocket') return 'websocket'
	return 'sse'
}
```

### `frontend/src/platform/connect/communicationsClient.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/platform/connect/communicationsClient.ts`
- Size bytes / Размер в байтах: `1202`
- Included characters / Включено символов: `1202`
- Truncated / Обрезано: `no`

```typescript
import { createClient } from '@connectrpc/connect'
import { createConnectTransport } from '@connectrpc/connect-web'
import type { Client } from '@connectrpc/connect'
import { ApiClient } from '../api/ApiClient'
import { CommunicationsService } from '../../gen/makosh/communications/v1/communications_pb'

let communicationsClient: Client<typeof CommunicationsService> | null = null

function createCommunicationsConnectClient(): Client<typeof CommunicationsService> {
	const apiClient = ApiClient.instance
	const secret = apiClient.getSecret()
	const transport = createConnectTransport({
		baseUrl: apiClient.getBaseUrl(),
		useBinaryFormat: false,
		fetch: (input, init) => {
			const headers = new Headers(init?.headers)
			headers.set('X-Макошь-Secret', secret)
			return fetch(input, {
				...init,
				headers
			})
		}
	})

	return createClient(CommunicationsService, transport)
}

export function getCommunicationsConnectClient(): Client<typeof CommunicationsService> {
	if (!communicationsClient) {
		communicationsClient = createCommunicationsConnectClient()
	}

	return communicationsClient
}

export function resetCommunicationsConnectClientForTests(): void {
	communicationsClient = null
}
```

### `frontend/src/platform/connect/signalHubClient.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/platform/connect/signalHubClient.ts`
- Size bytes / Размер в байтах: `1124`
- Included characters / Включено символов: `1124`
- Truncated / Обрезано: `no`

```typescript
import { createClient } from '@connectrpc/connect'
import { createConnectTransport } from '@connectrpc/connect-web'
import type { Client } from '@connectrpc/connect'
import { ApiClient } from '../api/ApiClient'
import { SignalHubService } from '../../gen/makosh/signal_hub/v1/signal_hub_pb'

let signalHubClient: Client<typeof SignalHubService> | null = null

function createSignalHubConnectClient(): Client<typeof SignalHubService> {
	const apiClient = ApiClient.instance
	const secret = apiClient.getSecret()
	const transport = createConnectTransport({
		baseUrl: apiClient.getBaseUrl(),
		useBinaryFormat: false,
		fetch: (input, init) => {
			const headers = new Headers(init?.headers)
			headers.set('X-Макошь-Secret', secret)
			return fetch(input, {
				...init,
				headers
			})
		}
	})

	return createClient(SignalHubService, transport)
}

export function getSignalHubConnectClient(): Client<typeof SignalHubService> {
	if (!signalHubClient) {
		signalHubClient = createSignalHubConnectClient()
	}

	return signalHubClient
}

export function resetSignalHubConnectClientForTests(): void {
	signalHubClient = null
}
```

### `frontend/src/platform/event-tracing/EventTracePanel.boundary.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/platform/event-tracing/EventTracePanel.boundary.test.ts`
- Size bytes / Размер в байтах: `683`
- Included characters / Включено символов: `683`
- Truncated / Обрезано: `no`

```typescript
import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('EventTracePanel boundary', () => {
  it('stays in platform event tracing ownership', () => {
    const source = readFileSync(new URL('./EventTracePanel.vue', import.meta.url), 'utf8')

    expect(source).toContain('EventTrace')
    expect(source).toContain('consumer_annotations')
    expect(source).toContain('dead_letters')
    expect(source).toContain('missing_parent_ids')
    expect(source).not.toContain("['telegram'")
    expect(source).not.toContain("['whatsapp'")
    expect(source).not.toContain('domains/telegram')
    expect(source).not.toContain('domains/whatsapp')
  })
})
```

### `frontend/src/platform/event-tracing/api.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/platform/event-tracing/api.test.ts`
- Size bytes / Размер в байтах: `1899`
- Included characters / Включено символов: `1899`
- Truncated / Обрезано: `no`

```typescript
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { ApiClient } from '../api/ApiClient'
import {
  fetchEventChildren,
  fetchEventTraceByCorrelationId,
  fetchEventTraceByEventId
} from './api'

describe('event tracing API', () => {
  beforeEach(() => {
    ApiClient.resetForTests()
    ApiClient.init('http://127.0.0.1:8080', 'test-secret')
  })

  afterEach(() => {
    vi.unstubAllGlobals()
    ApiClient.resetForTests()
  })

  it('fetches traces by event id and correlation id through platform endpoints', async () => {
    const fetchMock = vi.fn().mockImplementation(async () =>
      new Response(JSON.stringify(emptyTrace()), {
        status: 200,
        headers: { 'Content-Type': 'application/json' }
      })
    )
    vi.stubGlobal('fetch', fetchMock)

    await fetchEventTraceByEventId('event:v1:root', 25)
    await fetchEventTraceByCorrelationId('trace:root', 50)

    expect(fetchMock.mock.calls[0][0]).toBe(
      'http://127.0.0.1:8080/api/v1/events/event%3Av1%3Aroot/trace?limit=25'
    )
    expect(fetchMock.mock.calls[1][0]).toBe(
      'http://127.0.0.1:8080/api/v1/event-traces/trace%3Aroot?limit=50'
    )
  })

  it('fetches event children and clamps invalid limits', async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(JSON.stringify([]), {
        status: 200,
        headers: { 'Content-Type': 'application/json' }
      })
    )
    vi.stubGlobal('fetch', fetchMock)

    await fetchEventChildren('event:v1:parent', 5000)

    expect(fetchMock.mock.calls[0][0]).toBe(
      'http://127.0.0.1:8080/api/v1/events/event%3Av1%3Aparent/children?limit=1000'
    )
  })
})

function emptyTrace() {
  return {
    correlation_id: 'trace:root',
    root_event_ids: [],
    events: [],
    edges: [],
    orphan_event_ids: [],
    missing_parent_ids: [],
    consumer_annotations: [],
    dead_letters: []
  }
}
```

### `frontend/src/platform/event-tracing/api.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/platform/event-tracing/api.ts`
- Size bytes / Размер в байтах: `1468`
- Included characters / Включено символов: `1468`
- Truncated / Обрезано: `no`

```typescript
import { ApiClient } from '../api/ApiClient'
import type { EventTrace, StoredEventEnvelope } from './types'

export async function fetchEventTraceByEventId(
  eventId: string,
  limit = 1000
): Promise<EventTrace> {
  return ApiClient.instance.get<EventTrace>(
    `/api/v1/events/${encodeURIComponent(requiredIdentifier('eventId', eventId))}/trace${limitQuery(limit)}`,
    'Failed to fetch event trace'
  )
}

export async function fetchEventTraceByCorrelationId(
  correlationId: string,
  limit = 1000
): Promise<EventTrace> {
  return ApiClient.instance.get<EventTrace>(
    `/api/v1/event-traces/${encodeURIComponent(requiredIdentifier('correlationId', correlationId))}${limitQuery(limit)}`,
    'Failed to fetch event trace'
  )
}

export async function fetchEventChildren(
  eventId: string,
  limit = 1000
): Promise<StoredEventEnvelope[]> {
  return ApiClient.instance.get<StoredEventEnvelope[]>(
    `/api/v1/events/${encodeURIComponent(requiredIdentifier('eventId', eventId))}/children${limitQuery(limit)}`,
    'Failed to fetch event children'
  )
}

function requiredIdentifier(name: string, value: string): string {
  const trimmed = value.trim()
  if (trimmed.length === 0) {
    throw new Error(`${name} cannot be empty`)
  }
  return trimmed
}

function limitQuery(limit: number): string {
  const normalized = Number.isFinite(limit) ? Math.trunc(limit) : 1000
  const clamped = Math.min(Math.max(normalized, 1), 1000)
  return `?limit=${clamped}`
}
```

### `frontend/src/platform/event-tracing/index.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/platform/event-tracing/index.ts`
- Size bytes / Размер в байтах: `422`
- Included characters / Включено символов: `422`
- Truncated / Обрезано: `no`

```typescript
export {
  fetchEventChildren,
  fetchEventTraceByCorrelationId,
  fetchEventTraceByEventId
} from './api'
export {
  eventTraceQueryKeys,
  useEventChildrenQuery,
  useEventTraceByCorrelationIdQuery,
  useEventTraceByEventIdQuery
} from './queries'
export type {
  EventConsumerAnnotation,
  EventDeadLetterAnnotation,
  EventEnvelope,
  EventTrace,
  EventTraceEdge,
  JsonObject,
  StoredEventEnvelope
} from './types'
```

### `frontend/src/platform/event-tracing/queries.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/platform/event-tracing/queries.test.ts`
- Size bytes / Размер в байтах: `773`
- Included characters / Включено символов: `773`
- Truncated / Обрезано: `no`

```typescript
import { describe, expect, it } from 'vitest'
import { eventTraceQueryKeys } from './queries'

describe('event trace query keys', () => {
  it('uses provider-neutral event tracing keys', () => {
    expect(eventTraceQueryKeys.byEvent('event-1')).toEqual(['events', 'event-1', 'trace'])
    expect(eventTraceQueryKeys.byCorrelation('trace-1')).toEqual(['event-traces', 'trace-1'])
    expect(eventTraceQueryKeys.children('event-1')).toEqual(['events', 'event-1', 'children'])

    const flattened = [
      ...eventTraceQueryKeys.byEvent('event-1'),
      ...eventTraceQueryKeys.byCorrelation('trace-1'),
      ...eventTraceQueryKeys.children('event-1')
    ].join(' ')
    expect(flattened).not.toContain('telegram')
    expect(flattened).not.toContain('whatsapp')
  })
})
```

### `frontend/src/platform/event-tracing/queries.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/platform/event-tracing/queries.ts`
- Size bytes / Размер в байтах: `2696`
- Included characters / Включено символов: `2696`
- Truncated / Обрезано: `no`

```typescript
import { useQuery } from '@tanstack/vue-query'
import { computed, toValue, type MaybeRefOrGetter } from 'vue'
import {
  fetchEventChildren,
  fetchEventTraceByCorrelationId,
  fetchEventTraceByEventId
} from './api'
import type { EventTrace, StoredEventEnvelope } from './types'

export const eventTraceQueryKeys = {
  byEvent: (eventId: string) => ['events', eventId, 'trace'] as const,
  byCorrelation: (correlationId: string) => ['event-traces', correlationId] as const,
  children: (eventId: string) => ['events', eventId, 'children'] as const
}

export function useEventTraceByEventIdQuery(
  eventId: MaybeRefOrGetter<string | null | undefined>,
  limit: MaybeRefOrGetter<number> = 1000
) {
  return useQuery<EventTrace | null>({
    queryKey: computed(() => {
      const id = normalizeIdentifier(toValue(eventId))
      return id ? eventTraceQueryKeys.byEvent(id) : ['events', null, 'trace'] as const
    }),
    queryFn: async () => {
      const id = normalizeIdentifier(toValue(eventId))
      if (!id) return null
      return fetchEventTraceByEventId(id, toValue(limit))
    },
    enabled: computed(() => Boolean(normalizeIdentifier(toValue(eventId)))),
    staleTime: 10_000
  })
}

export function useEventTraceByCorrelationIdQuery(
  correlationId: MaybeRefOrGetter<string | null | undefined>,
  limit: MaybeRefOrGetter<number> = 1000
) {
  return useQuery<EventTrace | null>({
    queryKey: computed(() => {
      const id = normalizeIdentifier(toValue(correlationId))
      return id ? eventTraceQueryKeys.byCorrelation(id) : ['event-traces', null] as const
    }),
    queryFn: async () => {
      const id = normalizeIdentifier(toValue(correlationId))
      if (!id) return null
      return fetchEventTraceByCorrelationId(id, toValue(limit))
    },
    enabled: computed(() => Boolean(normalizeIdentifier(toValue(correlationId)))),
    staleTime: 10_000
  })
}

export function useEventChildrenQuery(
  eventId: MaybeRefOrGetter<string | null | undefined>,
  limit: MaybeRefOrGetter<number> = 1000
) {
  return useQuery<StoredEventEnvelope[]>({
    queryKey: computed(() => {
      const id = normalizeIdentifier(toValue(eventId))
      return id ? eventTraceQueryKeys.children(id) : ['events', null, 'children'] as const
    }),
    queryFn: async () => {
      const id = normalizeIdentifier(toValue(eventId))
      if (!id) return []
      return fetchEventChildren(id, toValue(limit))
    },
    enabled: computed(() => Boolean(normalizeIdentifier(toValue(eventId)))),
    staleTime: 10_000
  })
}

function normalizeIdentifier(value: string | null | undefined): string | null {
  const trimmed = value?.trim()
  return trimmed && trimmed.length > 0 ? trimmed : null
}
```

### `frontend/src/platform/event-tracing/types.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/platform/event-tracing/types.ts`
- Size bytes / Размер в байтах: `1125`
- Included characters / Включено символов: `1125`
- Truncated / Обрезано: `no`

```typescript
export type JsonObject = Record<string, unknown>

export type EventEnvelope = {
  event_id: string
  event_type: string
  schema_version: number
  occurred_at: string
  recorded_at: string
  source: JsonObject
  actor: JsonObject | null
  subject: JsonObject
  payload: unknown
  provenance: JsonObject
  causation_id: string | null
  correlation_id: string | null
}

export type StoredEventEnvelope = {
  position: number
  event: EventEnvelope
}

export type EventTraceEdge = {
  parent_event_id: string
  child_event_id: string
}

export type EventConsumerAnnotation = {
  event_id: string
  consumer_name: string
  status: string
  processed_at: string | null
  attempts: number | null
}

export type EventDeadLetterAnnotation = {
  event_id: string
  consumer_name: string | null
  reason: string
  failed_at: string | null
}

export type EventTrace = {
  correlation_id: string
  root_event_ids: string[]
  events: StoredEventEnvelope[]
  edges: EventTraceEdge[]
  orphan_event_ids: string[]
  missing_parent_ids: string[]
  consumer_annotations: EventConsumerAnnotation[]
  dead_letters: EventDeadLetterAnnotation[]
}
```
