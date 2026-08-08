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

- Chunk ID / ID чанка: `154-source-frontend-part-014`
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

### `frontend/src/integrations/yandexTelemost/types/yandexTelemost.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/integrations/yandexTelemost/types/yandexTelemost.ts`
- Size bytes / Размер в байтах: `6354`
- Included characters / Включено символов: `6354`
- Truncated / Обрезано: `no`

```typescript
export interface YandexTelemostCapabilityState {
  capability: string
  status: string
  source: string
  confidence: number
  evidence: Record<string, unknown>
}

export interface YandexTelemostLocalRecordingPolicy {
  macos: string
  linux: string
  windows: string
  ffmpeg_path_env: string
  ffmpeg_input_env: string
}

export interface YandexTelemostLocalRecordingManifest {
  state: string
  audio_format: 'mp3'
  recorder_boundary: string
  consent_required: boolean
  default_output_policy: string
  audio_device_policy: YandexTelemostLocalRecordingPolicy
}

export interface YandexTelemostSpeakerTimelinePolicy {
  state: string
  source: string
  reliability: string
  output_files: string[]
  role_in_transcription: string
}

export interface YandexTelemostCapabilitiesResponse {
  provider_kind: 'yandex_telemost_user'
  api_base_url: string
  web_origin: string
  capabilities: YandexTelemostCapabilityState[]
  recording_policy: YandexTelemostLocalRecordingManifest
  speaker_timeline_policy: YandexTelemostSpeakerTimelinePolicy
}

export interface YandexTelemostAccount {
  account_id: string
  provider_kind: 'yandex_telemost_user'
  display_name: string
  external_account_id: string
  lifecycle_state: string
  runtime_kind: string
  api_base_url: string
  token_secret_ref?: string | null
  join_webview_available: boolean
  local_recorder_available: boolean
  config: Record<string, unknown>
  created_at: string
  updated_at: string
}

export interface YandexTelemostAccountListResponse {
  items: YandexTelemostAccount[]
}

export interface YandexTelemostAccountSetupRequest {
  account_id: string
  display_name: string
  external_account_id: string
  oauth_token?: string
  oauth_token_ref?: string
  api_base_url?: string
  metadata?: Record<string, unknown>
}

export interface YandexTelemostAccountSetupResponse {
  account: YandexTelemostAccount
}

export interface YandexTelemostRuntimeStatus {
  account_id: string
  provider_kind: string
  lifecycle_state: string
  runtime_kind: string
  checked_at: string
  api_base_url: string
  authorized: boolean
  blockers: string[]
  capabilities: YandexTelemostCapabilityState[]
}

export interface YandexTelemostCohost {
  email: string
}

export interface YandexTelemostLiveStreamRequest {
  access_level?: string
  title?: string
  description?: string
}

export interface YandexTelemostLiveStreamResponse {
  watch_url?: string | null
  access_level?: string | null
  title?: string | null
  description?: string | null
}

export interface YandexTelemostConferenceCreateRequest {
  account_id: string
  waiting_room_level?: string
  live_stream?: YandexTelemostLiveStreamRequest
  cohosts?: YandexTelemostCohost[]
  is_auto_summarization_enabled?: boolean
  metadata?: Record<string, unknown>
}

export interface YandexTelemostConferenceUpdateRequest {
  waiting_room_level?: string
  live_stream?: YandexTelemostLiveStreamRequest
  cohosts?: YandexTelemostCohost[]
  is_auto_summarization_enabled?: boolean
  metadata?: Record<string, unknown>
}

export interface YandexTelemostConference {
  id: string
  join_url: string
  access_level?: string | null
  waiting_room_level?: string | null
  live_stream?: YandexTelemostLiveStreamResponse | null
  sip_uri_meeting?: string | null
  sip_uri_telemost?: string | null
  sip_id?: string | null
}

export interface YandexTelemostConferenceOperationResponse {
  account_id: string
  conference: YandexTelemostConference
  status: 'created' | 'observed' | 'updated'
}

export interface YandexTelemostCohostPage {
  cohosts: YandexTelemostCohost[]
}

export interface YandexTelemostWebviewManifestRequest {
  account_id: string
  conference_id?: string | null
  join_url: string
  display_name?: string | null
}

export interface YandexTelemostConferenceWebviewManifest {
  account_id: string
  conference_id?: string | null
  join_url: string
  target_origin: string
  provider_shape: string
  runtime_kind: string
  window_label: string
  opened_window: boolean
  focused_existing_window: boolean
  owner_visible: boolean
  hidden_headless_mode: string
  local_recording: YandexTelemostLocalRecordingManifest
  speaker_timeline: YandexTelemostSpeakerTimelinePolicy
}

export interface YandexTelemostRecordingIntentResponse {
  account_id: string
  conference_id?: string | null
  join_url: string
  consent_required: boolean
  source_of_truth: false
  local_recording: YandexTelemostLocalRecordingManifest
  speaker_timeline: YandexTelemostSpeakerTimelinePolicy
  tauri_commands: Record<string, string>
}

export interface YandexTelemostCompanionOpenRequest {
  account_id: string
  join_url: string
  conference_id?: string | null
  display_name?: string | null
}

export interface YandexTelemostCompanionManifest {
  account_id: string
  conference_id?: string | null
  join_url: string
  provider_shape: string
  runtime_kind: string
  window_label: string
  opened_window: boolean
  focused_existing_window: boolean
  owner_visible: boolean
  hidden_headless_mode: string
  allowed_hosts: string[]
  speaker_timeline: Record<string, unknown>
  recorder: Record<string, unknown>
}

export interface YandexTelemostRecordingSession {
  recording_session_id: string
  account_id: string
  conference_id?: string | null
  join_url: string
  window_label: string
  output_dir: string
  audio_path: string
  speaker_jsonl_path: string
  speaker_txt_path: string
  ffmpeg_pid?: number | null
  started_at_epoch_ms: number
  consent_attested: boolean
}

export interface YandexTelemostRecordingStopReceipt {
  recording_session_id: string
  account_id: string
  conference_id?: string | null
  audio_path: string
  speaker_jsonl_path: string
  speaker_txt_path: string
  stopped_at_epoch_ms: number
  state: string
}

export interface YandexTelemostRecordingBridgeRequest {
  account_id: string
  conference_id?: string | null
  join_url: string
  recording_session_id: string
  output_dir: string
  audio_path: string
  speaker_jsonl_path: string
  speaker_txt_path: string
  started_at_epoch_ms: number
  stopped_at_epoch_ms: number
  consent_attested: boolean
}

export interface YandexTelemostRecordingBridgeResponse {
  account_id: string
  conference_id?: string | null
  recording_session_id: string
  bundle_id: string
  bundle_root: string
  manifest_path: string
  follow_up_events: string[]
  radar_signal_kinds: string[]
}
```

### `frontend/src/integrations/zoom/api/zoom.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/integrations/zoom/api/zoom.test.ts`
- Size bytes / Размер в байтах: `17006`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```typescript
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { ApiClient } from '../../../platform/api/ApiClient'
import {
  authorizeZoomServerToServer,
  bridgeZoomMeeting,
  bridgeZoomRecording,
  bridgeZoomTranscript,
  cleanupZoomRetention,
  completeZoomOAuth,
  fetchZoomCallTranscript,
  fetchZoomAccounts,
  fetchZoomAuditEvents,
  fetchZoomCapabilities,
  fetchZoomRecordingImports,
  fetchZoomProviderCalls,
  fetchZoomRuntimeStatus,
  fetchZoomWebhookSubscriptionStatus,
  importZoomTranscriptFile,
  maintainZoomTokens,
  reconcileZoomWebhookSubscription,
  refreshZoomToken,
  removeZoomRecordingImport,
  removeZoomRuntime,
  removeZoomWebhookSubscription,
  setupZoomFixtureAccount,
  setupZoomLiveAccount,
  syncZoomRecordings,
  startZoomOAuth,
  startZoomRuntime,
  stopZoomRuntime,
} from './zoom'

describe('zoom integration API', () => {
  beforeEach(() => {
    ApiClient.resetForTests()
    ApiClient.init('http://127.0.0.1:8080', 'test-secret')
  })

  afterEach(() => {
    vi.unstubAllGlobals()
    ApiClient.resetForTests()
  })

  it('builds expected account and runtime routes', async () => {
    const ok = (body: unknown) =>
      new Response(JSON.stringify(body), {
        status: 200,
        headers: { 'Content-Type': 'application/json' },
      })
    const fetchMock = vi
      .fn()
      .mockResolvedValueOnce(ok({ version: '1.0', runtime_mode: 'fixture_plus_blocked_live', capabilities: [], planned_features: [], unsupported_features: [] }))
      .mockResolvedValueOnce(ok({ items: [] }))
      .mockResolvedValueOnce(ok({ account: { account_id: 'zoom-fixture-1' } }))
      .mockResolvedValueOnce(ok({ account: { account_id: 'zoom-live-1' } }))
      .mockResolvedValueOnce(ok({
        setup_id: 'zoom-oauth-setup-1',
        authorization_url: 'https://zoom.example.test/oauth/authorize',
        state: 'zoom-oauth-state',
        redirect_uri: 'http://127.0.0.1:8080/zoom/callback',
      }))
      .mockResolvedValueOnce(ok({
        account_id: 'zoom-live-1',
        provider_kind: 'zoom_user',
        auth_shape: 'oauth_user',
        lifecycle_state: 'authorized',
        runtime_kind: 'zoom_live_authorized_runtime',
        token_secret_ref: 'secret:zoom-token',
        client_secret_ref: 'secret:zoom-client-secret',
        secret_kind: 'oauth_token',
        store_kind: 'host_vault',
        authorized_at: '2026-06-27T10:00:00Z',
      }))
      .mockResolvedValueOnce(ok({
        account_id: 'zoom-s2s-1',
        provider_kind: 'zoom_server_to_server',
        auth_shape: 'server_to_server',
        lifecycle_state: 'authorized',
        runtime_kind: 'zoom_live_authorized_runtime',
        token_secret_ref: 'secret:zoom-s2s-token',
        client_secret_ref: 'secret:zoom-s2s-client-secret',
        secret_kind: 'oauth_token',
        store_kind: 'host_vault',
        authorized_at: '2026-06-27T10:00:00Z',
      }))
      .mockResolvedValueOnce(ok({
        account_id: 'zoom-live-1',
        provider_kind: 'zoom_user',
        auth_shape: 'oauth_user',
        token_secret_ref: 'secret:zoom-token',
        refreshed: true,
        refresh_strategy: 'oauth_refresh_token',
        status: 'refreshed',
        expires_at: '2026-06-27T11:00:00Z',
        checked_at: '2026-06-27T10:00:00Z',
        secret_kind: 'oauth_token',
        store_kind: 'host_vault',
      }))
      .mockResolvedValueOnce(ok({
        checked_count: 1,
        refreshed_count: 1,
        skipped_count: 0,
        failed_count: 0,
        refresh_expiring_within_seconds: 300,
        checked_at: '2026-06-27T10:00:00Z',
        items: [
          {
            account_id: 'zoom-live-1',
            provider_kind: 'zoom_user',
            auth_shape: 'oauth_user',
            status: 'refreshed',
            refreshed: true,
            expires_at: '2026-06-27T11:00:00Z',
          },
        ],
      }))
      .mockResolvedValueOnce(ok({
        account_id: 'zoom-live-1',
        user_id: 'me',
        from: '2026-06-01',
        to: '2026-06-30',
        meetings_seen: 1,
        meetings_recorded: 1,
        recordings_recorded: 2,
        media_downloads_recorded: 1,
        transcripts_recorded: 1,
        failures: [],
      }))
      .mockResolvedValueOnce(ok({
        account_id: 'zoom-live-1',
        provider_kind: 'zoom_user',
        auth_shape: 'oauth_user',
        checked_at: '2026-06-27T10:00:00Z',
        managed_subscription_id: 'zoom-subscription-1',
        subscriptions: [],
      }))
      .mockResolvedValueOnce(ok({
        account_id: 'zoom-live-1',
        provider_kind: 'zoom_user',
        auth_shape: 'oauth_user',
        status: 'created',
        checked_at: '2026-06-27T10:00:00Z',
        subscription: {
          subscription_id: 'zoom-subscription-1',
          subscription_name: 'Макошь Zoom Runtime',
          endpoint_url: 'https://makosh.example.test/api/v1/integrations/zoom/runtime-bridge/webhooks',
          event_types: ['meeting.started'],
        },
      }))
      .mockResolvedValueOnce(ok({
        account_id: 'zoom-live-1',
        provider_kind: 'zoom_user',
        auth_shape: 'oauth_user',
        removed: true,
        checked_at: '2026-06-27T10:00:00Z',
        subscription_id: 'zoom-subscription-1',
      }))
      .mockResolvedValueOnce(ok({ account_id: 'zoom-live-1', status: 'blocked' }))
      .mockResolvedValueOnce(ok({ account_id: 'zoom-live-1', status: 'blocked' }))
      .mockResolvedValueOnce(ok({ account_id: 'zoom-live-1', status: 'stopped' }))
      .mockResolvedValueOnce(ok({ account_id: 'zoom-live-1', removed: true }))
      .mockResolvedValueOnce(ok({ account_id: 'zoom-live-1', items: [] }))
      .mockResolvedValueOnce(ok({ account_id: 'zoom-live-1', items: [] }))
      .mockResolvedValueOnce(ok({
        account_id: 'zoom-live-1',
        attachment_id: 'attachment-1',
        blob_id: 'blob-1',
        removed: true,
        blob_metadata_removed: true,
        blob_file_removed: true,
        removed_at: '2026-06-27T10:00:00Z',
      }))
      .mockResolvedValueOnce(ok({
        account_id: 'zoom-live-1',
        checked_at: '2026-06-27T10:00:00Z',
        recordings_removed: 1,
        transcripts_removed: 1,
        items: [],
      }))
      .mockResolvedValueOnce(ok({ account_id: 'zoom-live-1', items: [] }))
    vi.stubGlobal('fetch', fetchMock)

    await fetchZoomCapabilities()
    await fetchZoomAccounts(true)
    await setupZoomFixtureAccount({
      account_id: 'zoom-fixture-1',
      display_name: 'Zoom Fixture',
      external_account_id: 'zoom-fixture-external',
    })
    await setupZoomLiveAccount({
      account_id: 'zoom-live-1',
      display_name: 'Zoom Live',
      external_account_id: 'zoom-live-external',
      auth_shape: 'oauth_user',
      client_id: 'zoom-client-id',
      token_secret_ref: 'secret:zoom-token',
      client_secret_ref: 'secret:zoom-client-secret',
      webhook_secret_ref: 'secret:zoom-webhook-secret',
    })
    await startZoomOAuth({
      account_id: 'zoom-live-1',
      display_name: 'Zoom Live',
      external_account_id: 'zoom-live-external',
      client_id: 'zoom-client-id',
      client_secret: 'zoom-client-secret',
      redirect_uri: 'http://127.0.0.1:8080/zoom/callback',
      scopes: ['meeting:read', 'recording:read'],
    })
    await completeZoomOAuth({
      setup_id: 'zoom-oauth-setup-1',
      state: 'zoom-oauth-state',
      authorization_code: 'zoom-auth-code',
    })
    await authorizeZoomServerToServer({
      account_id: 'zoom-s2s-1',
      client_id: 'zoom-s2s-client-id',
      client_secret: 'zoom-s2s-client-secret',
      zoom_account_id: 'zoom-provider-account',
    })
    await refreshZoomToken({ account_id: 'zoom-live-1', force: true })
    await maintainZoomTokens({ account_id: 'zoom-live-1', refresh_expiring_within_seconds: 300 })
    await syncZoomRecordings({
      account_id: 'zoom-live-1',
      from: '2026-06-01',
      to: '2026-06-30',
      page_size: 30,
      max_meetings: 100,
    })
    await fetchZoomWebhookSubscriptionStatus('zoom-live-1', 'https://api.zoom.example.test/v2')
    await reconcileZoomWebhookSubscription({
      account_id: 'zoom-live-1',
      endpoint_url: 'https://makosh.example.test/api/v1/integrations/zoom/runtime-bridge/webhooks',
      event_types: ['meeting.started'],
      api_base_url: 'https://api.zoom.example.test/v2',
    })
    await removeZoomWebhookSubscription({
      account_id: 'zoom-live-1',
      subscription_id: 'zoom-subscription-1',
      api_base_url: 'https://api.zoom.example.test/v2',
    })
    await fetchZoomRuntimeStatus(' zoom-live-1 ')
    await startZoomRuntime({ account_id: 'zoom-live-1', force: true })
    await stopZoomRuntime({ account_id: 'zoom-live-1', reason: 'test' })
    await removeZoomRuntime({ account_id: 'zoom-live-1', reason: 'test cleanup' })
    await fetchZoomRecordingImports(' zoom-live-1 ', 12)
    await fetchZoomAuditEvents(' zoom-live-1 ', 10)
    await removeZoomRecordingImport(' zoom-live-1 ', ' attachment-1 ', {
      reason: 'operator_removed_recording_import',
    })
    await cleanupZoomRetention(' zoom-live-1 ', {
      remove_recordings: true,
      remove_transcripts: true,
      limit: 100,
    })
    await fetchZoomRecordingImports(' zoom-live-1 ', 12)

    expect(fetchMock.mock.calls[0][0]).toContain('/api/v1/integrations/zoom/capabilities')
    expect(fetchMock.mock.calls[1][0]).toContain('/api/v1/integrations/zoom/accounts?include_removed=true')
    expect(fetchMock.mock.calls[2][0]).toContain('/api/v1/integrations/zoom/fixtures/accounts')
    expect(fetchMock.mock.calls[3][0]).toContain('/api/v1/integrations/zoom/accounts')
    expect(fetchMock.mock.calls[4][0]).toContain('/api/v1/integrations/zoom/oauth/start')
    expect(fetchMock.mock.calls[5][0]).toContain('/api/v1/integrations/zoom/oauth/complete')
    expect(fetchMock.mock.calls[6][0]).toContain(
      '/api/v1/integrations/zoom/oauth/server-to-server/authorize'
    )
    expect(fetchMock.mock.calls[7][0]).toContain('/api/v1/integrations/zoom/oauth/refresh')
    expect(fetchMock.mock.calls[8][0]).toContain('/api/v1/integrations/zoom/oauth/maintenance')
    expect(fetchMock.mock.calls[9][0]).toContain('/api/v1/integrations/zoom/provider-sync/recordings')
    expect(fetchMock.mock.calls[10][0]).toContain(
      '/api/v1/integrations/zoom/webhook-subscriptions/status?account_id=zoom-live-1&api_base_url=https%3A%2F%2Fapi.zoom.example.test%2Fv2'
    )
    expect(fetchMock.mock.calls[11][0]).toContain(
      '/api/v1/integrations/zoom/webhook-subscriptions/reconcile'
    )
    expect(fetchMock.mock.calls[12][0]).toContain(
      '/api/v1/integrations/zoom/webhook-subscriptions/remove'
    )
    expect(fetchMock.mock.calls[13][0]).toContain(
      '/api/v1/integrations/zoom/runtime/status?account_id=zoom-live-1'
    )
    expect(fetchMock.mock.calls[14][0]).toContain('/api/v1/integrations/zoom/runtime/start')
    expect(fetchMock.mock.calls[15][0]).toContain('/api/v1/integrations/zoom/runtime/stop')
    expect(fetchMock.mock.calls[16][0]).toContain('/api/v1/integrations/zoom/runtime/remove')
    expect(fetchMock.mock.calls[17][0]).toContain(
      '/api/v1/integrations/zoom/accounts/zoom-live-1/recording-imports?limit=12'
    )
    expect(fetchMock.mock.calls[18][0]).toContain(
      '/api/v1/integrations/zoom/accounts/zoom-live-1/audit-events?limit=10'
    )
    expect(fetchMock.mock.calls[19][0]).toContain(
      '/api/v1/integrations/zoom/accounts/zoom-live-1/recording-imports/attachment-1/remove'
    )
    expect(fetchMock.mock.calls[20][0]).toContain(
      '/api/v1/integrations/zoom/accounts/zoom-live-1/retention/prune'
    )
    expect(fetchMock.mock.calls[21][0]).toContain(
      '/api/v1/integrations/zoom/accounts/zoom-live-1/recording-imports?limit=12'
    )
    expect(JSON.parse(fetchMock.mock.calls[3][1].body as string)).toMatchObject({
      auth_shape: 'oauth_user',
      client_id: 'zoom-client-id',
      token_secret_ref: 'secret:zoom-token',
      client_secret
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `frontend/src/integrations/zoom/api/zoom.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/integrations/zoom/api/zoom.ts`
- Size bytes / Размер в байтах: `10342`
- Included characters / Включено символов: `10342`
- Truncated / Обрезано: `no`

```typescript
import { ApiClient } from '../../../platform/api/ApiClient'
import type {
  ZoomAccountListResponse,
  ZoomAccountSetupRequest,
  ZoomAccountSetupResponse,
  ZoomAuthorizationResult,
  ZoomAuditEventResponse,
  ZoomCallTranscriptResponse,
  ZoomCapabilitiesResponse,
  ZoomLiveAccountSetupRequest,
  ZoomMeetingIngestResult,
  ZoomMeetingObservationRequest,
  ZoomOAuthCompleteRequest,
  ZoomOAuthStartRequest,
  ZoomOAuthStartResponse,
  ZoomProviderCallListResponse,
  ZoomRecordingIngestResult,
  ZoomRecordingImportAuditResponse,
  ZoomRecordingImportRemoveRequest,
  ZoomRecordingImportRemoveResponse,
  ZoomRetentionCleanupRequest,
  ZoomRetentionCleanupResponse,
  ZoomRecordingObservationRequest,
  ZoomRecordingSyncRequest,
  ZoomRecordingSyncResult,
  ZoomRuntimeRemoveRequest,
  ZoomRuntimeRemoveResponse,
  ZoomRuntimeStartRequest,
  ZoomRuntimeStatus,
  ZoomRuntimeStopRequest,
  ZoomServerToServerAuthorizeRequest,
  ZoomTokenMaintenanceRequest,
  ZoomTokenMaintenanceResult,
  ZoomTokenRefreshRequest,
  ZoomTokenRefreshResult,
  ZoomTranscriptFileImportRequest,
  ZoomTranscriptFileImportResult,
  ZoomTranscriptIngestResult,
  ZoomTranscriptObservationRequest,
  ZoomWebhookSubscriptionReconcileRequest,
  ZoomWebhookSubscriptionReconcileResult,
  ZoomWebhookSubscriptionRemoveRequest,
  ZoomWebhookSubscriptionRemoveResult,
  ZoomWebhookSubscriptionStatusResult,
} from '../types/zoom'

export async function fetchZoomCapabilities(): Promise<ZoomCapabilitiesResponse> {
  return ApiClient.instance.get<ZoomCapabilitiesResponse>(
    '/api/v1/integrations/zoom/capabilities',
    'Zoom capabilities request failed'
  )
}

export async function fetchZoomAccounts(includeRemoved = false): Promise<ZoomAccountListResponse> {
  const params = new URLSearchParams()
  if (includeRemoved) params.set('include_removed', 'true')
  const suffix = params.toString() ? `?${params.toString()}` : ''
  return ApiClient.instance.get<ZoomAccountListResponse>(
    `/api/v1/integrations/zoom/accounts${suffix}`,
    'Zoom accounts request failed'
  )
}

export async function setupZoomFixtureAccount(
  request: ZoomAccountSetupRequest
): Promise<ZoomAccountSetupResponse> {
  return ApiClient.instance.post<ZoomAccountSetupResponse>(
    '/api/v1/integrations/zoom/fixtures/accounts',
    request,
    'Zoom fixture account setup failed'
  )
}

export async function setupZoomLiveAccount(
  request: ZoomLiveAccountSetupRequest
): Promise<ZoomAccountSetupResponse> {
  return ApiClient.instance.post<ZoomAccountSetupResponse>(
    '/api/v1/integrations/zoom/accounts',
    request,
    'Zoom account setup failed'
  )
}

export async function startZoomOAuth(
  request: ZoomOAuthStartRequest
): Promise<ZoomOAuthStartResponse> {
  return ApiClient.instance.post<ZoomOAuthStartResponse>(
    '/api/v1/integrations/zoom/oauth/start',
    request,
    'Zoom OAuth start failed'
  )
}

export async function completeZoomOAuth(
  request: ZoomOAuthCompleteRequest
): Promise<ZoomAuthorizationResult> {
  return ApiClient.instance.post<ZoomAuthorizationResult>(
    '/api/v1/integrations/zoom/oauth/complete',
    request,
    'Zoom OAuth completion failed'
  )
}

export async function authorizeZoomServerToServer(
  request: ZoomServerToServerAuthorizeRequest
): Promise<ZoomAuthorizationResult> {
  return ApiClient.instance.post<ZoomAuthorizationResult>(
    '/api/v1/integrations/zoom/oauth/server-to-server/authorize',
    request,
    'Zoom Server-to-Server authorization failed'
  )
}

export async function refreshZoomToken(
  request: ZoomTokenRefreshRequest
): Promise<ZoomTokenRefreshResult> {
  return ApiClient.instance.post<ZoomTokenRefreshResult>(
    '/api/v1/integrations/zoom/oauth/refresh',
    request,
    'Zoom token refresh failed'
  )
}

export async function maintainZoomTokens(
  request: ZoomTokenMaintenanceRequest = {}
): Promise<ZoomTokenMaintenanceResult> {
  return ApiClient.instance.post<ZoomTokenMaintenanceResult>(
    '/api/v1/integrations/zoom/oauth/maintenance',
    request,
    'Zoom token maintenance failed'
  )
}

export async function syncZoomRecordings(
  request: ZoomRecordingSyncRequest
): Promise<ZoomRecordingSyncResult> {
  return ApiClient.instance.post<ZoomRecordingSyncResult>(
    '/api/v1/integrations/zoom/provider-sync/recordings',
    request,
    'Zoom recording sync failed'
  )
}

export async function fetchZoomWebhookSubscriptionStatus(
  accountId: string,
  apiBaseUrl?: string | null
): Promise<ZoomWebhookSubscriptionStatusResult> {
  const params = new URLSearchParams({ account_id: accountId.trim() })
  if (apiBaseUrl?.trim()) params.set('api_base_url', apiBaseUrl.trim())
  return ApiClient.instance.get<ZoomWebhookSubscriptionStatusResult>(
    `/api/v1/integrations/zoom/webhook-subscriptions/status?${params.toString()}`,
    'Zoom webhook subscription status request failed'
  )
}

export async function reconcileZoomWebhookSubscription(
  request: ZoomWebhookSubscriptionReconcileRequest
): Promise<ZoomWebhookSubscriptionReconcileResult> {
  return ApiClient.instance.post<ZoomWebhookSubscriptionReconcileResult>(
    '/api/v1/integrations/zoom/webhook-subscriptions/reconcile',
    request,
    'Zoom webhook subscription reconcile failed'
  )
}

export async function removeZoomWebhookSubscription(
  request: ZoomWebhookSubscriptionRemoveRequest
): Promise<ZoomWebhookSubscriptionRemoveResult> {
  return ApiClient.instance.post<ZoomWebhookSubscriptionRemoveResult>(
    '/api/v1/integrations/zoom/webhook-subscriptions/remove',
    request,
    'Zoom webhook subscription removal failed'
  )
}

export async function fetchZoomRuntimeStatus(accountId: string): Promise<ZoomRuntimeStatus> {
  const params = new URLSearchParams({ account_id: accountId.trim() })
  return ApiClient.instance.get<ZoomRuntimeStatus>(
    `/api/v1/integrations/zoom/runtime/status?${params.toString()}`,
    'Zoom runtime status request failed'
  )
}

export async function fetchZoomRecordingImports(
  accountId: string,
  limit = 20
): Promise<ZoomRecordingImportAuditResponse> {
  const params = new URLSearchParams({ limit: String(limit) })
  return ApiClient.instance.get<ZoomRecordingImportAuditResponse>(
    `/api/v1/integrations/zoom/accounts/${encodeURIComponent(accountId.trim())}/recording-imports?${params.toString()}`,
    'Zoom recording imports request failed'
  )
}

export async function removeZoomRecordingImport(
  accountId: string,
  attachmentId: string,
  request: ZoomRecordingImportRemoveRequest = {}
): Promise<ZoomRecordingImportRemoveResponse> {
  return ApiClient.instance.post<ZoomRecordingImportRemoveResponse>(
    `/api/v1/integrations/zoom/accounts/${encodeURIComponent(accountId.trim())}/recording-imports/${encodeURIComponent(attachmentId.trim())}/remove`,
    request,
    'Zoom recording import removal failed'
  )
}

export async function fetchZoomAuditEvents(
  accountId: string,
  limit = 25
): Promise<ZoomAuditEventResponse> {
  const params = new URLSearchParams({ limit: String(limit) })
  return ApiClient.instance.get<ZoomAuditEventResponse>(
    `/api/v1/integrations/zoom/accounts/${encodeURIComponent(accountId.trim())}/audit-events?${params.toString()}`,
    'Zoom audit events request failed'
  )
}

export async function cleanupZoomRetention(
  accountId: string,
  request: ZoomRetentionCleanupRequest = {}
): Promise<ZoomRetentionCleanupResponse> {
  return ApiClient.instance.post<ZoomRetentionCleanupResponse>(
    `/api/v1/integrations/zoom/accounts/${encodeURIComponent(accountId.trim())}/retention/prune`,
    request,
    'Zoom retention cleanup failed'
  )
}

export async function fetchZoomProviderCalls(
  accountId?: string,
  limit = 20
): Promise<ZoomProviderCallListResponse> {
  const params = new URLSearchParams()
  params.set('limit', String(limit))
  params.set('provider', 'zoom')
  if (accountId?.trim()) params.set('account_id', accountId.trim())
  return ApiClient.instance.get<ZoomProviderCallListResponse>(
    `/api/v1/calls?${params.toString()}`,
    'Zoom provider calls request failed'
  )
}

export async function fetchZoomCallTranscript(callId: string): Promise<ZoomCallTranscriptResponse> {
  return ApiClient.instance.get<ZoomCallTranscriptResponse>(
    `/api/v1/calls/${encodeURIComponent(callId)}/transcript`,
    'Zoom call transcript request failed'
  )
}

export async function startZoomRuntime(
  request: ZoomRuntimeStartRequest
): Promise<ZoomRuntimeStatus> {
  return ApiClient.instance.post<ZoomRuntimeStatus>(
    '/api/v1/integrations/zoom/runtime/start',
    request,
    'Zoom runtime start failed'
  )
}

export async function stopZoomRuntime(request: ZoomRuntimeStopRequest): Promise<ZoomRuntimeStatus> {
  return ApiClient.instance.post<ZoomRuntimeStatus>(
    '/api/v1/integrations/zoom/runtime/stop',
    request,
    'Zoom runtime stop failed'
  )
}

export async function removeZoomRuntime(
  request: ZoomRuntimeRemoveRequest
): Promise<ZoomRuntimeRemoveResponse> {
  return ApiClient.instance.post<ZoomRuntimeRemoveResponse>(
    '/api/v1/integrations/zoom/runtime/remove',
    request,
    'Zoom runtime remove failed'
  )
}

export async function bridgeZoomMeeting(
  request: ZoomMeetingObservationRequest
): Promise<ZoomMeetingIngestResult> {
  return ApiClient.instance.post<ZoomMeetingIngestResult>(
    '/api/v1/integrations/zoom/runtime-bridge/meetings',
    request,
    'Zoom meeting bridge ingest failed'
  )
}

export async function bridgeZoomRecording(
  request: ZoomRecordingObservationRequest
): Promise<ZoomRecordingIngestResult> {
  return ApiClient.instance.post<ZoomRecordingIngestResult>(
    '/api/v1/integrations/zoom/runtime-bridge/recordings',
    request,
    'Zoom recording bridge ingest failed'
  )
}

export async function bridgeZoomTranscript(
  request: ZoomTranscriptObservationRequest
): Promise<ZoomTranscriptIngestResult> {
  return ApiClient.instance.post<ZoomTranscriptIngestResult>(
    '/api/v1/integrations/zoom/runtime-bridge/transcripts',
    request,
    'Zoom transcript bridge ingest failed'
  )
}

export async function importZoomTranscriptFile(
  request: ZoomTranscriptFileImportRequest
): Promise<ZoomTranscriptFileImportResult> {
  return ApiClient.instance.post<ZoomTranscriptFileImportResult>(
    '/api/v1/integrations/zoom/runtime-bridge/transcript-files',
    request,
    'Zoom transcript file import failed'
  )
}
```

### `frontend/src/integrations/zoom/components/ZoomBridgeLab.boundary.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/integrations/zoom/components/ZoomBridgeLab.boundary.test.ts`
- Size bytes / Размер в байтах: `824`
- Included characters / Включено символов: `824`
- Truncated / Обрезано: `no`

```typescript
import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('ZoomBridgeLab boundary', () => {
  it('uses the existing bridge mutations instead of raw fetch calls', () => {
    const source = readFileSync(new URL('./ZoomBridgeLab.vue', import.meta.url), 'utf8')

    expect(source).toContain('useBridgeZoomMeetingMutation')
    expect(source).toContain('useBridgeZoomRecordingMutation')
    expect(source).toContain('useBridgeZoomTranscriptMutation')
    expect(source).toContain('useImportZoomTranscriptFileMutation')
    expect(source).toContain('handleBridgeMeeting')
    expect(source).toContain('handleBridgeRecording')
    expect(source).toContain('handleBridgeTranscript')
    expect(source).toContain('handleImportTranscriptFile')
    expect(source).not.toContain('fetch(')
  })
})
```

### `frontend/src/integrations/zoom/components/ZoomObservedCallsPanel.boundary.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/integrations/zoom/components/ZoomObservedCallsPanel.boundary.test.ts`
- Size bytes / Размер в байтах: `836`
- Included characters / Включено символов: `836`
- Truncated / Обрезано: `no`

```typescript
import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('ZoomObservedCallsPanel boundary', () => {
  it('loads projected call and transcript evidence through the query layer', () => {
    const source = readFileSync(new URL('./ZoomObservedCallsPanel.vue', import.meta.url), 'utf8')

    expect(source).toContain('useZoomProviderCallsQuery')
    expect(source).toContain('useZoomCallTranscriptQuery')
    expect(source).toContain('extractZoomRecordingRefs')
    expect(source).toContain('formatZoomTranscriptProvenance')
    expect(source).toContain("t('Observed calls')")
    expect(source).toContain("t('Transcript evidence')")
    expect(source).toContain("t('Recording references')")
    expect(source).toContain("t('Transcript provenance')")
    expect(source).not.toContain('fetch(')
  })
})
```

### `frontend/src/integrations/zoom/components/ZoomSettingsPanel.boundary.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/integrations/zoom/components/ZoomSettingsPanel.boundary.test.ts`
- Size bytes / Размер в байтах: `3018`
- Included characters / Включено символов: `3018`
- Truncated / Обрезано: `no`

```typescript
import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('ZoomSettingsPanel boundary', () => {
  it('wires the full Zoom setup and credential maintenance flow through query mutations', () => {
    const source = readFileSync(new URL('./ZoomSettingsPanel.vue', import.meta.url), 'utf8')
    const recordingMaintenanceSource = readFileSync(
      new URL('./ZoomRecordingMaintenancePanel.vue', import.meta.url),
      'utf8'
    )

    expect(source).toContain('useStartZoomOAuthMutation')
    expect(source).toContain('useCompleteZoomOAuthMutation')
    expect(source).toContain('useAuthorizeZoomServerToServerMutation')
    expect(source).toContain('useRefreshZoomTokenMutation')
    expect(source).toContain('useMaintainZoomTokensMutation')
    expect(source).toContain('useZoomCapabilitiesQuery')
    expect(source).toContain('token_rotation_policy')
    expect(source).toContain('planned_features')
    expect(source).toContain("import ZoomAuditEventsPanel from './ZoomAuditEventsPanel.vue'")
    expect(source).toContain("import ZoomBridgeLab from './ZoomBridgeLab.vue'")
    expect(source).toContain("import ZoomRecordingMaintenancePanel from './ZoomRecordingMaintenancePanel.vue'")
    expect(source).toContain("import ZoomObservedCallsPanel from './ZoomObservedCallsPanel.vue'")
    expect(source).toContain("import ZoomRecordingImportsPanel from './ZoomRecordingImportsPanel.vue'")
    expect(source).toContain('<ZoomAuditEventsPanel :selected-account="selectedAccount" />')
    expect(source).toContain('<ZoomBridgeLab :selected-account="selectedAccount" />')
    expect(source).toContain('<ZoomRecordingMaintenancePanel :selected-account="selectedAccount" />')
    expect(source).toContain('<ZoomObservedCallsPanel :selected-account="selectedAccount" />')
    expect(source).toContain('<ZoomRecordingImportsPanel :selected-account="selectedAccount" />')
    expect(source).toContain('window.open(response.authorization_url')
    expect(recordingMaintenanceSource).toContain('useSyncZoomRecordingsMutation')
    expect(recordingMaintenanceSource).toContain('useCleanupZoomRetentionMutation')
    expect(recordingMaintenanceSource).toContain('Manual recording sync')
    expect(recordingMaintenanceSource).toContain('handleSyncZoomRecordings')
    expect(recordingMaintenanceSource).toContain('Sync cloud recordings')
    expect(recordingMaintenanceSource).toContain('handleCleanupZoomRetention')
    expect(recordingMaintenanceSource).toContain('Run retention cleanup')
    expect(recordingMaintenanceSource).toContain('privacy.zoom_remote_transcript_download_enabled')
    expect(recordingMaintenanceSource).toContain('privacy.zoom_remote_recording_download_enabled')
    expect(recordingMaintenanceSource).toContain('privacy.zoom_recording_import_retention_days')
    expect(recordingMaintenanceSource).toContain('privacy.zoom_transcript_retention_days')
    expect(source).not.toContain('fetch(')
    expect(recordingMaintenanceSource).not.toContain('fetch(')
  })
})
```

### `frontend/src/integrations/zoom/components/zoomEvidence.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/integrations/zoom/components/zoomEvidence.test.ts`
- Size bytes / Размер в байтах: `1349`
- Included characters / Включено символов: `1345`
- Truncated / Обрезано: `no`

```typescript
import { describe, expect, it } from 'vitest'

import { extractZoomRecordingRefs, formatZoomTranscriptProvenance } from './zoomEvidence'

describe('zoomEvidence', () => {
  it('extracts only valid recording refs from call metadata', () => {
    const recordingRefs = extractZoomRecordingRefs({
      recording_refs: [
        {
          recording_id: 'rec-1',
          recording_type: 'shared_screen_with_speaker_view',
          file_extension: 'MP4',
        },
        {
          recording_id: '',
          recording_type: 'audio_only',
        },
        null,
      ],
    })

    expect(recordingRefs).toEqual([
      {
        recording_id: 'rec-1',
        recording_type: 'shared_screen_with_speaker_view',
        file_extension: 'MP4',
      },
    ])
  })

  it('formats transcript provenance as stable sorted json', () => {
    const formatted = formatZoomTranscriptProvenance({
      metadata: {
        topic: 'Weekly review',
        account_id: 'zoom-1',
      },
      provider: 'zoom',
    })

    expect(formatted).toBe(`{
  "metadata": {
    "account_id": "zoom-1",
    "topic": "Weekly review"
  },
  "provider": "zoom"
}`)
  })

  it('returns dash when provenance is missing or empty', () => {
    expect(formatZoomTranscriptProvenance(null)).toBe('—')
    expect(formatZoomTranscriptProvenance({})).toBe('—')
  })
})
```

### `frontend/src/integrations/zoom/components/zoomEvidence.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/integrations/zoom/components/zoomEvidence.ts`
- Size bytes / Размер в байтах: `1482`
- Included characters / Включено символов: `1478`
- Truncated / Обрезано: `no`

```typescript
import type { ZoomRecordingRef } from '../types/zoom'

type UnknownRecord = Record<string, unknown>

export function extractZoomRecordingRefs(metadata: UnknownRecord | null | undefined): ZoomRecordingRef[] {
  const recordingRefs = metadata?.recording_refs
  if (!Array.isArray(recordingRefs)) return []

  return recordingRefs.filter(isZoomRecordingRef)
}

export function formatZoomTranscriptProvenance(provenance: unknown): string {
  if (!isUnknownRecord(provenance)) return '—'

  const normalized = sortObject(provenance)
  const entries = Object.entries(normalized)
  if (entries.length === 0) return '—'

  return JSON.stringify(normalized, null, 2)
}

function isZoomRecordingRef(value: unknown): value is ZoomRecordingRef {
  return isUnknownRecord(value) && typeof value.recording_id === 'string' && value.recording_id.trim().length > 0
}

function isUnknownRecord(value: unknown): value is UnknownRecord {
  return typeof value === 'object' && value !== null && !Array.isArray(value)
}

function sortObject(value: UnknownRecord): UnknownRecord {
  return Object.fromEntries(
    Object.entries(value)
      .sort(([left], [right]) => left.localeCompare(right))
      .map(([key, entry]) => {
        if (Array.isArray(entry)) {
          return [key, entry.map((item) => (isUnknownRecord(item) ? sortObject(item) : item))]
        }
        if (isUnknownRecord(entry)) {
          return [key, sortObject(entry)]
        }
        return [key, entry]
      })
  )
}
```

### `frontend/src/integrations/zoom/queries/useZoomRuntimeQuery.boundary.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/integrations/zoom/queries/useZoomRuntimeQuery.boundary.test.ts`
- Size bytes / Размер в байтах: `953`
- Included characters / Включено символов: `953`
- Truncated / Обрезано: `no`

```typescript
import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('zoom runtime query mutation invalidation boundary', () => {
  it('invalidates derived provider call caches for local ingest and provider sync mutations', () => {
    const source = readFileSync(new URL('./useZoomRuntimeQuery.ts', import.meta.url), 'utf8')

    expect(source).toContain('function invalidateZoomDerived')
    expect(source).toContain("queryClient.invalidateQueries({ queryKey: zoomQueryKeys.providerCalls })")
    expect(source).toContain("queryClient.invalidateQueries({ queryKey: zoomQueryKeys.callTranscript })")
    expect(source).toContain('useBridgeZoomMeetingMutation')
    expect(source).toContain('useBridgeZoomRecordingMutation')
    expect(source).toContain('useBridgeZoomTranscriptMutation')
    expect(source).toContain('useImportZoomTranscriptFileMutation')
    expect(source).toContain('useSyncZoomRecordingsMutation')
  })
})
```

### `frontend/src/integrations/zoom/queries/useZoomRuntimeQuery.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/integrations/zoom/queries/useZoomRuntimeQuery.ts`
- Size bytes / Размер в байтах: `13152`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```typescript
import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, toValue, type MaybeRefOrGetter } from 'vue'
import {
  authorizeZoomServerToServer,
  bridgeZoomMeeting,
  bridgeZoomRecording,
  bridgeZoomTranscript,
  cleanupZoomRetention,
  completeZoomOAuth,
  fetchZoomCallTranscript,
  fetchZoomAccounts,
  fetchZoomAuditEvents,
  fetchZoomCapabilities,
  fetchZoomRecordingImports,
  removeZoomRecordingImport,
  fetchZoomProviderCalls,
  fetchZoomRuntimeStatus,
  fetchZoomWebhookSubscriptionStatus,
  importZoomTranscriptFile,
  maintainZoomTokens,
  reconcileZoomWebhookSubscription,
  refreshZoomToken,
  removeZoomRuntime,
  removeZoomWebhookSubscription,
  setupZoomFixtureAccount,
  setupZoomLiveAccount,
  syncZoomRecordings,
  startZoomOAuth,
  startZoomRuntime,
  stopZoomRuntime,
} from '../api/zoom'
import type {
  ZoomAccount,
  ZoomAccountSetupRequest,
  ZoomAccountSetupResponse,
  ZoomAuditEventItem,
  ZoomAuthorizationResult,
  ZoomCallTranscript,
  ZoomCapabilitiesResponse,
  ZoomLiveAccountSetupRequest,
  ZoomMeetingIngestResult,
  ZoomMeetingObservationRequest,
  ZoomOAuthCompleteRequest,
  ZoomOAuthStartRequest,
  ZoomOAuthStartResponse,
  ZoomProviderCall,
  ZoomRecordingIngestResult,
  ZoomRecordingImportAuditItem,
  ZoomRecordingImportRemoveRequest,
  ZoomRecordingImportRemoveResponse,
  ZoomRecordingObservationRequest,
  ZoomRecordingSyncRequest,
  ZoomRecordingSyncResult,
  ZoomRetentionCleanupRequest,
  ZoomRetentionCleanupResponse,
  ZoomRuntimeRemoveRequest,
  ZoomRuntimeRemoveResponse,
  ZoomRuntimeStartRequest,
  ZoomRuntimeStatus,
  ZoomRuntimeStopRequest,
  ZoomServerToServerAuthorizeRequest,
  ZoomTokenMaintenanceRequest,
  ZoomTokenMaintenanceResult,
  ZoomTokenRefreshRequest,
  ZoomTokenRefreshResult,
  ZoomTranscriptFileImportRequest,
  ZoomTranscriptFileImportResult,
  ZoomTranscriptIngestResult,
  ZoomTranscriptObservationRequest,
  ZoomWebhookSubscriptionReconcileRequest,
  ZoomWebhookSubscriptionReconcileResult,
  ZoomWebhookSubscriptionRemoveRequest,
  ZoomWebhookSubscriptionRemoveResult,
  ZoomWebhookSubscriptionStatusResult,
} from '../types/zoom'
import { zoomQueryKeys } from './zoomQueryKeys'

export function useZoomCapabilitiesQuery() {
  return useQuery<ZoomCapabilitiesResponse>({
    queryKey: zoomQueryKeys.capabilities,
    queryFn: fetchZoomCapabilities,
  })
}

export function useZoomAccountsQuery(includeRemoved: MaybeRefOrGetter<boolean> = false) {
  return useQuery<ZoomAccount[]>({
    queryKey: computed(() => [...zoomQueryKeys.accounts, toValue(includeRemoved)]),
    queryFn: async () => (await fetchZoomAccounts(toValue(includeRemoved))).items,
  })
}

export function useZoomRuntimeStatusQuery(
  accountId: MaybeRefOrGetter<string | null | undefined>
) {
  return useQuery<ZoomRuntimeStatus | null>({
    queryKey: computed(() => [...zoomQueryKeys.runtimeStatus, toValue(accountId) ?? 'none']),
    queryFn: async () => {
      const value = toValue(accountId)
      if (!value) return null
      return fetchZoomRuntimeStatus(value)
    },
    enabled: computed(() => Boolean(toValue(accountId))),
  })
}

export function useZoomWebhookSubscriptionStatusQuery(
  accountId: MaybeRefOrGetter<string | null | undefined>,
  apiBaseUrl: MaybeRefOrGetter<string | null | undefined> = null
) {
  return useQuery<ZoomWebhookSubscriptionStatusResult | null>({
    queryKey: computed(() => [
      ...zoomQueryKeys.webhookSubscriptions,
      'status',
      toValue(accountId) ?? 'none',
      toValue(apiBaseUrl) ?? 'default',
    ]),
    queryFn: async () => {
      const value = toValue(accountId)
      if (!value) return null
      return fetchZoomWebhookSubscriptionStatus(value, toValue(apiBaseUrl))
    },
    enabled: computed(() => Boolean(toValue(accountId))),
  })
}

export function useZoomProviderCallsQuery(
  accountId: MaybeRefOrGetter<string | null | undefined>,
  limit: MaybeRefOrGetter<number> = 20
) {
  return useQuery<ZoomProviderCall[]>({
    queryKey: computed(() => [
      ...zoomQueryKeys.providerCalls,
      toValue(accountId) ?? 'none',
      toValue(limit),
    ]),
    queryFn: async () => {
      const value = toValue(accountId)
      if (!value) return []
      return (await fetchZoomProviderCalls(value, toValue(limit))).items
    },
    enabled: computed(() => Boolean(toValue(accountId))),
  })
}

export function useZoomRecordingImportsQuery(
  accountId: MaybeRefOrGetter<string | null | undefined>,
  limit: MaybeRefOrGetter<number> = 20
) {
  return useQuery<ZoomRecordingImportAuditItem[]>({
    queryKey: computed(() => [
      ...zoomQueryKeys.recordingImports,
      toValue(accountId) ?? 'none',
      toValue(limit),
    ]),
    queryFn: async () => {
      const value = toValue(accountId)
      if (!value) return []
      return (await fetchZoomRecordingImports(value, toValue(limit))).items
    },
    enabled: computed(() => Boolean(toValue(accountId))),
  })
}

export function useRemoveZoomRecordingImportMutation(
  accountId: MaybeRefOrGetter<string | null | undefined>
) {
  const queryClient = useQueryClient()
  return useMutation<
    ZoomRecordingImportRemoveResponse,
    Error,
    { attachmentId: string; request?: ZoomRecordingImportRemoveRequest }
  >({
    mutationFn: async ({ attachmentId, request }) => {
      const value = toValue(accountId)
      if (!value) {
        throw new Error('Zoom account id is required to remove an imported recording')
      }
      return removeZoomRecordingImport(value, attachmentId, request ?? {})
    },
    onSuccess: () => {
      invalidateZoomDerived(queryClient)
    },
  })
}

export function useCleanupZoomRetentionMutation(
  accountId: MaybeRefOrGetter<string | null | undefined>
) {
  const queryClient = useQueryClient()
  return useMutation<ZoomRetentionCleanupResponse, Error, ZoomRetentionCleanupRequest | undefined>(
    {
      mutationFn: async (request) => {
        const value = toValue(accountId)
        if (!value) {
          throw new Error('Zoom account id is required to run retention cleanup')
        }
        return cleanupZoomRetention(value, request ?? {})
      },
      onSuccess: () => {
        invalidateZoomDerived(queryClient)
      },
    }
  )
}

export function useZoomAuditEventsQuery(
  accountId: MaybeRefOrGetter<string | null | undefined>,
  limit: MaybeRefOrGetter<number> = 25
) {
  return useQuery<ZoomAuditEventItem[]>({
    queryKey: computed(() => [
      ...zoomQueryKeys.auditEvents,
      toValue(accountId) ?? 'none',
      toValue(limit),
    ]),
    queryFn: async () => {
      const value = toValue(accountId)
      if (!value) return []
      return (await fetchZoomAuditEvents(value, toValue(limit))).items
    },
    enabled: computed(() => Boolean(toValue(accountId))),
  })
}

export function useZoomCallTranscriptQuery(
  callId: MaybeRefOrGetter<string | null | undefined>
) {
  return useQuery<ZoomCallTranscript | null>({
    queryKey: computed(() => [
      ...zoomQueryKeys.callTranscript,
      toValue(callId) ?? 'none',
    ]),
    queryFn: async () => {
      const value = toValue(callId)
      if (!value) return null
      return (await fetchZoomCallTranscript(value)).transcript
    },
    enabled: computed(() => Boolean(toValue(callId))),
  })
}

function invalidateZoomRuntime(queryClient: ReturnType<typeof useQueryClient>) {
  queryClient.invalidateQueries({ queryKey: zoomQueryKeys.accounts })
  queryClient.invalidateQueries({ queryKey: zoomQueryKeys.capabilities })
  queryClient.invalidateQueries({ queryKey: zoomQueryKeys.runtimeStatus })
  queryClient.invalidateQueries({ queryKey: zoomQueryKeys.webhookSubscriptions })
}

function invalidateZoomDerived(queryClient: ReturnType<typeof useQueryClient>) {
  queryClient.invalidateQueries({ queryKey: zoomQueryKeys.providerCalls })
  queryClient.invalidateQueries({ queryKey: zoomQueryKeys.callTranscript })
  queryClient.invalidateQueries({ queryKey: zoomQueryKeys.recordingImports })
  queryClient.invalidateQueries({ queryKey: zoomQueryKeys.auditEvents })
}

export function useSetupZoomFixtureAccountMutation() {
  const queryClient = useQueryClient()
  return useMutation<ZoomAccountSetupResponse, Error, ZoomAccountSetupRequest>({
    mutationFn: setupZoomFixtureAccount,
    onSuccess: () => invalidateZoomRuntime(queryClient),
  })
}

export function useSetupZoomLiveAccountMutation() {
  const queryClient = useQueryClient()
  return useMutation<ZoomAccountSetupResponse, Error, ZoomLiveAccountSetupRequest>({
    mutationFn: setupZoomLiveAccount,
    onSuccess: () => invalidateZoomRuntime(queryClient),
  })
}

export function useStartZoomOAuthMutation() {
  const queryClient = useQueryClient()
  return useMutation<ZoomOAuthStartResponse, Error, ZoomOAuthStartRequest>({
    mutationFn: startZoomOAuth,
    onSuccess: () => invalidateZoomRuntime(queryClient),
  })
}

export function useCompleteZoomOAuthMutation() {
  const queryClient = useQueryClient()
  return useMutation<ZoomAuthorizationResult, Error, ZoomOAuthCompleteRequest>({
    mutationFn: completeZoomOAuth,
    onSuccess: () => invalidateZoomRuntime(queryClient),
  })
}

export function useAuthorizeZoomServerToServerMutation() {
  const queryClient = useQueryClient()
  return useMutation<ZoomAuthorizationResult, Error, ZoomServerToServerAuthorizeRequest>({
    mutationFn: authorizeZoomServerToServer,
    onSuccess: () => invalidateZoomRuntime(queryClient),
  })
}

export function useRefreshZoomTokenMutation() {
  const queryClient = useQueryClient()
  return useMutation<ZoomTokenRefreshResult, Error, ZoomTokenRefreshRequest>({
    mutationFn: refreshZoomToken,
    onSuccess: () => invalidateZoomRuntime(queryClient),
  })
}

export function useMaintainZoomTokensMutation() {
  const queryClient = useQueryClient()
  return useMutation<ZoomTokenMaintenanceResult, Error, ZoomTokenMaintenanceRequest | undefined>({
    mutationFn: (request) => maintainZoomTokens(request ?? {}),
    onSuccess: () => invalidateZoomRuntime(queryClient),
  })
}

export function useReconcileZoomWebhookSubscriptionMutation() {
  const queryClient = useQueryClient()
  return useMutation<
    ZoomWebhookSubscriptionReconcileResult,
    Error,
    ZoomWebhookSubscriptionReconcileRequest
  >({
    mutationFn: reconcileZoomWebhookSubscription,
    onSuccess: () => invalidateZoomRuntime(queryClient),
  })
}

export function useRemoveZoomWebhookSubscriptionMutation() {
  const queryClient = useQueryClient()
  return useMutation<
    ZoomWebhookSubscriptionRemoveResult,
    Error,
    ZoomWebhookSubscriptionRemoveRequest
  >({
    mutationFn: removeZoomWebhookSubscription,
    onSuccess: () => invalidateZoomRuntime(queryClient),
  })
}

export function useStartZoomRuntimeMutation() {
  const queryClient = useQueryClient()
  return useMutation<ZoomRuntimeStatus, Error, ZoomRuntimeStartRequest>({
    mutationFn: startZoomRuntime,
    onSuccess: () => invalidateZoomRuntime(queryClient),
  })
}

export function useStopZoomRuntimeMutation() {
  const queryClient = useQueryClient()
  return useMutation<ZoomRuntimeStatus, Error, ZoomRuntimeStopRequest>({
    mutationFn: stopZoomRuntime,
    onSuccess: () => invalidateZoomRuntime(queryClient),
  })
}

export function useRemoveZoomRuntimeMutation() {
  const queryClient = useQueryClient()
  return useMutation<ZoomRuntimeRemoveResponse, Error, ZoomRuntimeRemoveRequest>({
    mutationFn: removeZoomRuntime,
    onSuccess: () => invalidateZoomRuntime(queryClient),
  })
}

export function useBridgeZoomMeetingMutation() {
  const queryClient = useQueryClient()
  return useMutation<ZoomMeetingIngestResult, Error, ZoomMeetingObservationRequest>({
    mutationFn: bridgeZoomMeeting,
    onSuccess: () => {
      invalidateZoomRuntime(queryClient)
      invalidateZoomDerived(queryClient)
    },
  })
}

export function useBridgeZoomRecordingMutation() {
  const queryClient = useQueryClient()
  return useMutation<ZoomRecordingIngestResult, Error, ZoomRecordingObservationRequest>({
    mutationFn: bridgeZ
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `frontend/src/integrations/zoom/queries/zoomQueryKeys.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/integrations/zoom/queries/zoomQueryKeys.test.ts`
- Size bytes / Размер в байтах: `527`
- Included characters / Включено символов: `527`
- Truncated / Обрезано: `no`

```typescript
import { describe, expect, it } from 'vitest'
import { zoomQueryKeys } from './zoomQueryKeys'

describe('zoom query keys', () => {
  it('keeps every key under the integration zoom namespace', () => {
    for (const queryKey of Object.values(zoomQueryKeys)) {
      expect(queryKey[0]).toBe('integrations')
      expect(queryKey[1]).toBe('zoom')
    }
  })

  it('declares a dedicated recording import audit key', () => {
    expect(zoomQueryKeys.recordingImports).toEqual(['integrations', 'zoom', 'recording-imports'])
  })
})
```

### `frontend/src/integrations/zoom/queries/zoomQueryKeys.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/integrations/zoom/queries/zoomQueryKeys.ts`
- Size bytes / Размер в байтах: `1019`
- Included characters / Включено символов: `1019`
- Truncated / Обрезано: `no`

```typescript
export const zoomQueryKeys = {
  accounts: ['integrations', 'zoom', 'accounts'] as const,
  capabilities: ['integrations', 'zoom', 'capabilities'] as const,
  oauth: ['integrations', 'zoom', 'oauth'] as const,
  runtimeStatus: ['integrations', 'zoom', 'runtime', 'status'] as const,
  webhookSubscriptions: ['integrations', 'zoom', 'webhook-subscriptions'] as const,
  providerCalls: ['integrations', 'zoom', 'provider-calls'] as const,
  callTranscript: ['integrations', 'zoom', 'provider-call-transcript'] as const,
  recordingImports: ['integrations', 'zoom', 'recording-imports'] as const,
  auditEvents: ['integrations', 'zoom', 'audit-events'] as const,
  meetingsBridge: ['integrations', 'zoom', 'runtime-bridge', 'meetings'] as const,
  recordingsBridge: ['integrations', 'zoom', 'runtime-bridge', 'recordings'] as const,
  transcriptsBridge: ['integrations', 'zoom', 'runtime-bridge', 'transcripts'] as const,
  transcriptFilesBridge: ['integrations', 'zoom', 'runtime-bridge', 'transcript-files'] as const,
}
```

### `frontend/src/integrations/zoom/types/zoom.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/integrations/zoom/types/zoom.ts`
- Size bytes / Размер в байтах: `11884`
- Included characters / Включено символов: `11884`
- Truncated / Обрезано: `no`

```typescript
export type ZoomAuthShape = 'fixture' | 'oauth_user' | 'server_to_server'

export interface ZoomAccountSetupRequest {
  account_id: string
  display_name: string
  external_account_id: string
  account_email?: string | null
  metadata?: Record<string, unknown>
}

export interface ZoomLiveAccountSetupRequest extends ZoomAccountSetupRequest {
  auth_shape?: Exclude<ZoomAuthShape, 'fixture'>
  client_id: string
  token_secret_ref?: string | null
  client_secret_ref?: string | null
  webhook_secret_ref?: string | null
}

export interface ZoomAccount {
  account_id: string
  provider_kind: string
  display_name: string
  external_account_id: string
  auth_shape: string
  lifecycle_state: string
  runtime_kind: string
  account_email?: string | null
  config: Record<string, unknown>
  created_at: string
  updated_at: string
}

export interface ZoomAccountSetupResponse {
  account: ZoomAccount
}

export interface ZoomAccountListResponse {
  items: ZoomAccount[]
}

export interface ZoomRuntimeStatus {
  account_id: string
  provider_kind: string
  runtime_kind: string
  status: string
  healthy: boolean
  auth_shape: string
  live_runtime_available: boolean
  recording_ingest_available: boolean
  transcript_ingest_available: boolean
  runtime_blockers: string[]
  last_error?: string | null
  checked_at: string
  metadata: Record<string, unknown>
}

export interface ZoomRuntimeStartRequest {
  account_id: string
  force?: boolean
}

export interface ZoomRuntimeStopRequest {
  account_id: string
  reason?: string | null
}

export interface ZoomRuntimeRemoveRequest {
  account_id: string
  reason?: string | null
}

export interface ZoomRuntimeRemoveResponse {
  account_id: string
  provider_kind: string
  removed: boolean
  removed_at: string
}

export interface ZoomCapabilityStatus {
  capability: string
  category: string
  status: string
  action_class: string
  confirmation_required: boolean
  reason: string
}

export interface ZoomCapabilitiesResponse {
  version: string
  runtime_mode: string
  capabilities: ZoomCapabilityStatus[]
  planned_features: string[]
  unsupported_features: string[]
}

export interface ZoomOAuthStartRequest {
  account_id: string
  display_name: string
  external_account_id: string
  account_email?: string | null
  client_id: string
  client_secret?: string | null
  client_secret_ref?: string | null
  webhook_secret_ref?: string | null
  redirect_uri: string
  app_return_url?: string | null
  scopes?: string[]
  authorization_endpoint?: string | null
  token_endpoint?: string | null
  metadata?: Record<string, unknown>
}

export interface ZoomOAuthStartResponse {
  setup_id: string
  authorization_url: string
  state: string
  redirect_uri: string
}

export interface ZoomOAuthCompleteRequest {
  setup_id: string
  state: string
  authorization_code: string
  external_account_id?: string | null
}

export interface ZoomServerToServerAuthorizeRequest {
  account_id: string
  client_id: string
  client_secret?: string | null
  client_secret_ref?: string | null
  zoom_account_id?: string | null
  token_endpoint?: string | null
  metadata?: Record<string, unknown>
}

export interface ZoomAuthorizationResult {
  account_id: string
  provider_kind: string
  auth_shape: string
  lifecycle_state: string
  runtime_kind: string
  token_secret_ref: string
  client_secret_ref?: string | null
  secret_kind: string
  store_kind: string
  authorized_at: string
}

export interface ZoomTokenRefreshRequest {
  account_id: string
  force?: boolean
  refresh_expiring_within_seconds?: number | null
}

export interface ZoomTokenRefreshResult {
  account_id: string
  provider_kind: string
  auth_shape: string
  token_secret_ref: string
  refreshed: boolean
  refresh_strategy: string
  status: string
  expires_at: string
  checked_at: string
  secret_kind: string
  store_kind: string
}

export interface ZoomTokenMaintenanceRequest {
  account_id?: string | null
  force?: boolean
  refresh_expiring_within_seconds?: number | null
}

export interface ZoomTokenMaintenanceItem {
  account_id: string
  provider_kind: string
  auth_shape: string
  status: string
  refreshed: boolean
  expires_at?: string | null
  error?: string | null
}

export interface ZoomTokenMaintenanceResult {
  checked_count: number
  refreshed_count: number
  skipped_count: number
  failed_count: number
  refresh_expiring_within_seconds: number
  checked_at: string
  items: ZoomTokenMaintenanceItem[]
}

export interface ZoomRecordingSyncRequest {
  account_id: string
  user_id?: string | null
  from: string
  to: string
  page_size?: number | null
  max_meetings?: number | null
  api_base_url?: string | null
}

export interface ZoomRecordingSyncFailure {
  meeting_id: string
  step: string
  error: string
}

export interface ZoomRecordingSyncResult {
  account_id: string
  user_id: string
  from: string
  to: string
  meetings_seen: number
  meetings_recorded: number
  recordings_recorded: number
  media_downloads_recorded: number
  transcripts_recorded: number
  failures: ZoomRecordingSyncFailure[]
}

export interface ZoomRecordingImportAuditItem {
  attachment_id: string
  account_id: string
  meeting_id?: string | null
  meeting_uuid?: string | null
  recording_id?: string | null
  filename?: string | null
  content_type: string
  size_bytes: number
  sha256: string
  source?: string | null
  scan_status: string
  scan_summary?: string | null
  storage_kind: string
  storage_path: string
  retention_mode: string
  retention_days: number
  expires_at?: string | null
  created_at: string
}

export interface ZoomRecordingImportAuditResponse {
  account_id: string
  items: ZoomRecordingImportAuditItem[]
}

export interface ZoomRecordingImportRemoveRequest {
  reason?: string | null
}

export interface ZoomRecordingImportRemoveResponse {
  account_id: string
  attachment_id: string
  blob_id: string
  recording_id?: string | null
  removed: boolean
  blob_metadata_removed: boolean
  blob_file_removed: boolean
  removed_at: string
}

export interface ZoomRetentionCleanupRequest {
  remove_recordings?: boolean
  remove_transcripts?: boolean
  limit?: number
}

export interface ZoomRetentionCleanupItem {
  evidence_kind: string
  entity_id: string
  call_id?: string | null
  meeting_id?: string | null
  recording_id?: string | null
  transcript_id?: string | null
  expires_at?: string | null
  removed_at: string
}

export interface ZoomRetentionCleanupResponse {
  account_id: string
  checked_at: string
  recordings_removed: number
  transcripts_removed: number
  items: ZoomRetentionCleanupItem[]
}

export interface ZoomAuditEventItem {
  position: number
  event_id: string
  event_type: string
  occurred_at: string
  subject_kind?: string | null
  subject_entity_id?: string | null
  correlation_id?: string | null
  source: Record<string, unknown>
  subject: Record<string, unknown>
  payload: Record<string, unknown>
  provenance: Record<string, unknown>
}

export interface ZoomAuditEventResponse {
  account_id: string
  items: ZoomAuditEventItem[]
}

export interface ZoomWebhookSubscriptionStatusRequest {
  account_id: string
  api_base_url?: string | null
}

export interface ZoomWebhookSubscriptionReconcileRequest {
  account_id: string
  endpoint_url: string
  subscription_name?: string | null
  event_types?: string[]
  api_base_url?: string | null
}

export interface ZoomWebhookSubscriptionRemoveRequest {
  account_id: string
  subscription_id?: string | null
  api_base_url?: string | null
}

export interface ZoomWebhookSubscription {
  subscription_id: string
  subscription_name: string
  endpoint_url: string
  event_types: string[]
}

export interface ZoomWebhookSubscriptionStatusResult {
  account_id: string
  provider_kind: string
  auth_shape: string
  checked_at: string
  managed_subscription_id?: string | null
  subscriptions: ZoomWebhookSubscription[]
}

export interface ZoomWebhookSubscriptionReconcileResult {
  account_id: string
  provider_kind: string
  auth_shape: string
  status: string
  checked_at: string
  subscription: ZoomWebhookSubscription
}

export interface ZoomWebhookSubscriptionRemoveResult {
  account_id: string
  provider_kind: string
  auth_shape: string
  removed: boolean
  checked_at: string
  subscription_id?: string | null
}

export interface ZoomParticipantSnapshot {
  participant_id?: string | null
  display_name?: string | null
  email?: string | null
  joined_at?: string | null
  left_at?: string | null
  metadata?: Record<string, unknown>
}

export interface ZoomRecordingRef {
  recording_id: string
  recording_type?: string | null
  download_ref?: string | null
  file_extension?: string | null
  file_size_bytes?: number | null
  recorded_at?: string | null
  metadata?: Record<string, unknown>
}

export interface ZoomMeetingObservationRequest {
  observation_id?: string | null
  account_id: string
  meeting_id: string
  meeting_uuid?: string | null
  topic?: string | null
  host_email?: string | null
  join_url?: string | null
  started_at?: string | null
  ended_at?: string | null
  duration_seconds?: number | null
  participants?: ZoomParticipantSnapshot[]
  recording_refs?: ZoomRecordingRef[]
  transcript_ref?: string | null
  metadata?: Record<string, unknown>
  causation_id?: string | null
  correlation_id?: string | null
}

export interface ZoomMeetingIngestResult {
  call_id: string
  account_id: string
  meeting_id: string
  event_id: string
  status: string
}

export interface ZoomRecordingObservationRequest {
  observation_id?: string | null
  account_id: string
  meeting_id: string
  recording: ZoomRecordingRef
  metadata?: Record<string, unknown>
  causation_id?: string | null
  correlation_id?: string | null
}

export interface ZoomRecordingIngestResult {
  account_id: string
  meeting_id: string
  recording_id: string
  event_id: string
  status: string
}

export interface ZoomTranscriptObservationRequest {
  observation_id?: string | null
  transcript_id: string
  account_id: string
  meeting_id: string
  meeting_uuid?: string | null
  source_recording_ref?: string | null
  language_code?: string | null
  transcript_text: string
  segments?: unknown[]
  metadata?: Record<string, unknown>
  causation_id?: string | null
  correlation_id?: string | null
}

export interface ZoomTranscriptIngestResult {
  transcript_id: string
  call_id: string
  account_id: string
  meeting_id: string
  event_id: string
  status: string
}

export interface ZoomTranscriptFileImportRequest {
  observation_id?: string | null
  transcript_id: string
  account_id: string
  meeting_id: string
  meeting_uuid?: string | null
  source_recording_ref?: string | null
  language_code?: string | null
  file_name?: string | null
  content_type?: string | null
  file_text: string
  metadata?: Record<string, unknown>
  causation_id?: string | null
  correlation_id?: string | null
}

export interface ZoomTranscriptFileImportResult extends ZoomTranscriptIngestResult {
  import_format: string
  parsed_segment_count: number
}

export interface ZoomProviderCall {
  call_id: string
  account_id: string
  provider_call_id: string
  provider_chat_id: string
  direction: string
  call_state: string
  started_at?: string | null
  ended_at?: string | null
  transcription_policy_id?: string | null
  metadata: Record<string, unknown>
  created_at: string
  updated_at: string
}

export interface ZoomProviderCallListResponse {
  items: ZoomProviderCall[]
}

export interface ZoomCallTranscript {
  transcript_id: string
  call_id: string
  account_id: string
  provider_chat_id: string
  transcript_status: string
  stt_provider: string
  source_audio_ref?: string | null
  language_code?: string | null
  transcript_text: string
  segments: unknown
  provenance: unknown
  created_at: string
  updated_at: string
}

export interface ZoomCallTranscriptResponse {
  transcript: ZoomCallTranscript | null
}
```

### `frontend/src/main.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/main.ts`
- Size bytes / Размер в байтах: `1724`
- Included characters / Включено символов: `1724`
- Truncated / Обрезано: `no`

```typescript
import { createApp } from 'vue'
import { createPinia } from 'pinia'
import { QueryClient, VueQueryPlugin } from '@tanstack/vue-query'
import App from './app/App.vue'
import router from './app/router'
import { initializeApiClient } from './platform/bootstrap/api'
import { initializeRealtime } from './platform/bootstrap/realtime'
import { loadFrontendConfig } from './platform/config/env'
import { useRealtimeStatusStore } from './shared/stores/realtimeStatus'
import './style.css'
import './styles/surfaces.css'
import './styles/theme-classes.css'

const app = createApp(App)
const pinia = createPinia()
const queryClient = new QueryClient()
let realtimeClient: ReturnType<typeof initializeRealtime> | null = null

app.use(pinia)
app.use(VueQueryPlugin, { queryClient })
app.use(router)

try {
	const config = loadFrontendConfig()
	const realtimeStatus = useRealtimeStatusStore(pinia)
	initializeApiClient(config)
	realtimeClient = initializeRealtime(config, queryClient, {
		onEventObserved: realtimeStatus.observeRealtimeEvent,
		onLaggedObserved: realtimeStatus.observeRealtimeLag,
		onStatus: realtimeStatus.setRealtimeStatus
	})
	realtimeStatus.setReconnectHandler(() => realtimeClient?.reconnect())
} catch (error) {
	document.body.innerHTML = `<main class="startup-error"><h1>Макошь cannot start</h1><p>${escapeHtml(error instanceof Error ? error.message : 'Unknown startup error')}</p></main>`
	throw error
}

app.mount('#app')

window.addEventListener('beforeunload', () => {
	realtimeClient?.disconnect()
})

function escapeHtml(value: string): string {
	return value
		.replaceAll('&', '&amp;')
		.replaceAll('<', '&lt;')
		.replaceAll('>', '&gt;')
		.replaceAll('"', '&quot;')
		.replaceAll("'", '&#39;')
}
```

### `frontend/src/platform/api/ApiClient.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/platform/api/ApiClient.ts`
- Size bytes / Размер в байтах: `2680`
- Included characters / Включено символов: `2680`
- Truncated / Обрезано: `no`

```typescript
import type { ApiError } from './types'

export class ApiClient {
	private baseUrl: string
	private secret: string

	constructor(baseUrl: string, secret: string) {
		this.baseUrl = baseUrl.replace(/\/+$/, '')
		this.secret = secret
	}

	private async request<T>(
		method: string,
		path: string,
		body?: unknown,
		fallbackMessage?: string
	): Promise<T> {
		const url = `${this.baseUrl}${path}`
		const headers: Record<string, string> = {
			'Content-Type': 'application/json',
			'X-Макошь-Secret': this.secret
		}

		const res = await fetch(url, {
			method,
			headers,
			body: body !== undefined ? JSON.stringify(body) : undefined
		})

		if (!res.ok) {
			let errorBody: string | undefined
			try {
				errorBody = await res.text()
			} catch {
				// ignore parse error
			}
			const err: ApiError = {
				message: errorBody ?? fallbackMessage ?? `${method} request failed`,
				status: res.status
			}
			throw err
		}

		// Handle 204 No Content
		if (res.status === 204) {
			return undefined as T
		}

		return res.json() as Promise<T>
	}

	async get<T>(path: string, fallbackMessage = 'GET request failed'): Promise<T> {
		return this.request<T>('GET', path, undefined, fallbackMessage)
	}

	async post<T>(path: string, body: unknown, fallbackMessage = 'POST request failed'): Promise<T> {
		return this.request<T>('POST', path, body, fallbackMessage)
	}

	async put<T>(path: string, body: unknown, fallbackMessage = 'PUT request failed'): Promise<T> {
		return this.request<T>('PUT', path, body, fallbackMessage)
	}

	async patch<T>(path: string, body: unknown, fallbackMessage = 'PATCH request failed'): Promise<T> {
		return this.request<T>('PATCH', path, body, fallbackMessage)
	}

	async delete<T>(path: string, fallbackMessage = 'DELETE request failed'): Promise<T> {
		return this.request<T>('DELETE', path, undefined, fallbackMessage)
	}

	async deleteWithBody<T>(path: string, body: unknown, fallbackMessage = 'DELETE request failed'): Promise<T> {
		return this.request<T>('DELETE', path, body, fallbackMessage)
	}

	getBaseUrl(): string {
		return this.baseUrl
	}

	getSecret(): string {
		return this.secret
	}

	private static _instance: ApiClient | null = null

	static get instance(): ApiClient {
		if (!ApiClient._instance) {
			throw new Error('ApiClient not initialized. Call ApiClient.init() first.')
		}
		return ApiClient._instance
	}

	static init(baseUrl: string, secret: string): ApiClient {
		if (secret.trim().length === 0) {
			throw new Error('X-Макошь-Secret cannot be empty')
		}

		ApiClient._instance = new ApiClient(baseUrl, secret)
		return ApiClient._instance
	}

	static resetForTests(): void {
		ApiClient._instance = null
	}
}
```

### `frontend/src/platform/api/index.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/platform/api/index.ts`
- Size bytes / Размер в байтах: `99`
- Included characters / Включено символов: `99`
- Truncated / Обрезано: `no`

```typescript
export { ApiClient } from './ApiClient'
export type { ApiError, PaginatedResponse } from './types'
```

### `frontend/src/platform/api/types.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/platform/api/types.ts`
- Size bytes / Размер в байтах: `173`
- Included characters / Включено символов: `173`
- Truncated / Обрезано: `no`

```typescript
export type ApiError = {
	message: string
	status?: number
	code?: string
}

export type PaginatedResponse<T> = {
	data: T[]
	total: number
	offset: number
	limit: number
}
```

### `frontend/src/platform/bootstrap/api.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/platform/bootstrap/api.test.ts`
- Size bytes / Размер в байтах: `596`
- Included characters / Включено символов: `596`
- Truncated / Обрезано: `no`

```typescript
import { beforeEach, describe, expect, it } from 'vitest'
import { ApiClient } from '../api/ApiClient'
import { initializeApiClient } from './api'

describe('initializeApiClient', () => {
	beforeEach(() => {
		ApiClient.resetForTests()
	})

	it('initializes the singleton from config', () => {
		initializeApiClient({
			apiBaseUrl: 'http://127.0.0.1:8080',
			apiSecret: 'dev-secret',
			sseUrl: 'http://127.0.0.1:8080/api/events/stream',
			webSocketUrl: 'ws://127.0.0.1:8080/api/events/ws',
			realtimeTransport: 'websocket'
		})

		expect(ApiClient.instance).toBeInstanceOf(ApiClient)
	})
})
```

### `frontend/src/platform/bootstrap/api.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/platform/bootstrap/api.ts`
- Size bytes / Размер в байтах: `233`
- Included characters / Включено символов: `233`
- Truncated / Обрезано: `no`

```typescript
import { ApiClient } from '../api/ApiClient'
import type { FrontendConfig } from '../config/env'

export function initializeApiClient(config: FrontendConfig): ApiClient {
	return ApiClient.init(config.apiBaseUrl, config.apiSecret)
}
```

### `frontend/src/platform/bootstrap/businessCommunicationOwnership.boundary.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/platform/bootstrap/businessCommunicationOwnership.boundary.test.ts`
- Size bytes / Размер в байтах: `1891`
- Included characters / Включено символов: `1891`
- Truncated / Обрезано: `no`

```typescript
import { describe, expect, it } from 'vitest'
import { readFileSync, readdirSync, statSync } from 'node:fs'
import { join, relative } from 'node:path'

const root = new URL('../..', import.meta.url)

function filesUnder(path: string): string[] {
	const dir = new URL(path, root)
	const output: string[] = []
	for (const entry of readdirSync(dir)) {
		const fullPath = join(dir.pathname, entry)
		const stat = statSync(fullPath)
		if (stat.isDirectory()) {
			output.push(...filesUnder(relative(root.pathname, fullPath)))
		} else if (/\.(ts|vue)$/.test(entry)) {
			output.push(fullPath)
		}
	}
	return output
}

function readAll(paths: string[]): string {
	return paths.map((path) => readFileSync(path, 'utf8')).join('\n')
}

describe('business communication hooks ownership boundary', () => {
	it('keeps shared communication modules DTO-only', () => {
		const source = readAll(filesUnder('shared/communications'))

		expect(source).not.toMatch(/\/api\/v1\/communications/)
		expect(source).not.toMatch(/\buseQuery\b/)
		expect(source).not.toMatch(/\bqueryKey\b/)
		expect(source).not.toMatch(/\[\s*['"]communications['"]/)
		expect(source).not.toMatch(/\bfetch\(/)
	})

	it('keeps integration modules out of Communications business read models', () => {
		const source = readAll(filesUnder('integrations'))

		expect(source).not.toMatch(/shared\/communications\/.*Business/)
		expect(source).not.toMatch(/\[\s*['"]communications['"]/)
		expect(source).not.toMatch(/\/api\/v1\/communications\/(conversations|messages|search|topics)/)
		expect(source).not.toMatch(/MessageThread|ChatList|MediaGallery|RawEvidence|ReplyChain|ForwardChain|Reactions|Topics/)
	})

	it('keeps Communications domain out of provider-control endpoints', () => {
		const source = readAll(filesUnder('domains/communications'))

		expect(source).not.toMatch(/\/api\/v1\/integrations\/(telegram|whatsapp|mail)/)
	})
})
```

### `frontend/src/platform/bootstrap/realtime.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/platform/bootstrap/realtime.test.ts`
- Size bytes / Размер в байтах: `15067`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```typescript
import { afterEach, describe, expect, it, vi } from 'vitest'
import { initializeRealtime, handleRealtimeEvent } from './realtime'
import type { RealtimeClientOptions } from './realtime'
import type { SseClientOptions, WebSocketClientOptions } from '../sse'

describe('realtime bootstrap', () => {
  afterEach(() => {
    vi.unstubAllGlobals()
  })

  it('creates a protected SSE client and connects it', () => {
    const connect = vi.fn()
    const queryClient = { invalidateQueries: vi.fn() }
    const onStatus = vi.fn()
    let capturedOptions: RealtimeClientOptions | null = null

    const client = initializeRealtime(
      {
        apiBaseUrl: 'http://127.0.0.1:8080',
        apiSecret: 'test-secret',
        sseUrl: 'http://127.0.0.1:8080/api/events/stream',
        webSocketUrl: 'ws://127.0.0.1:8080/api/events/ws',
        realtimeTransport: 'sse'
      },
      queryClient,
      {
        onStatus,
        createClient: (options) => {
          capturedOptions = options
          return { connect, disconnect: vi.fn(), reconnect: vi.fn() }
        }
      }
    )

    expect(client).toBeDefined()
    expect(connect).toHaveBeenCalledOnce()
    expect(capturedOptions).not.toBeNull()
    const options = capturedOptions as unknown as SseClientOptions
    expect(options).toMatchObject({
      url: 'http://127.0.0.1:8080/api/events/stream',
      longPollUrl: 'http://127.0.0.1:8080/api/v1/events',
      secret: 'test-secret'
    })
    options.onMessage?.({ id: '10', event: 'event', data: '{}' })
    expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
      queryKey: ['communications-list']
    })
    options.onStatus?.({ transport: 'sse', state: 'connected' })
    expect(onStatus).toHaveBeenCalledWith({ transport: 'sse', state: 'connected' })
  })

  it('starts WebSocket transport first and falls back to SSE when it disconnects', () => {
    const queryClient = { invalidateQueries: vi.fn() }
    const createdOptions: RealtimeClientOptions[] = []
    const connectedUrls: string[] = []

    initializeRealtime(
      {
        apiBaseUrl: 'http://127.0.0.1:8080',
        apiSecret: 'test-secret',
        sseUrl: 'http://127.0.0.1:8080/api/events/stream',
        webSocketUrl: 'ws://127.0.0.1:8080/api/events/ws',
        realtimeTransport: 'websocket'
      },
      queryClient,
      {
        createClient: (options) => {
          createdOptions.push(options)
          return {
            connect: () => connectedUrls.push(options.url),
            disconnect: vi.fn(),
            reconnect: vi.fn()
          }
        }
      }
    )

    expect(createdOptions[0].url).toBe('ws://127.0.0.1:8080/api/events/ws')
    expect(connectedUrls).toEqual(['ws://127.0.0.1:8080/api/events/ws'])

    ;(createdOptions[0] as WebSocketClientOptions).onStatus?.({
      transport: 'websocket',
      state: 'disconnected',
      error: 'WebSocket reconnect attempts exhausted'
    })

    expect(createdOptions[1].url).toBe('http://127.0.0.1:8080/api/events/stream')
    expect(connectedUrls).toEqual([
      'ws://127.0.0.1:8080/api/events/ws',
      'http://127.0.0.1:8080/api/events/stream'
    ])
  })

  it('allows manual reconnect to prefer the primary WebSocket transport again', () => {
    const createdOptions: RealtimeClientOptions[] = []
    const connectedUrls: string[] = []
    const disconnectedUrls: string[] = []

    const client = initializeRealtime(
      {
        apiBaseUrl: 'http://127.0.0.1:8080',
        apiSecret: 'test-secret',
        sseUrl: 'http://127.0.0.1:8080/api/events/stream',
        webSocketUrl: 'ws://127.0.0.1:8080/api/events/ws',
        realtimeTransport: 'websocket'
      },
      { invalidateQueries: vi.fn() },
      {
        createClient: (options) => {
          createdOptions.push(options)
          return {
            connect: () => connectedUrls.push(options.url),
            disconnect: () => disconnectedUrls.push(options.url),
            reconnect: vi.fn()
          }
        }
      }
    )

    ;(createdOptions[0] as WebSocketClientOptions).onStatus?.({
      transport: 'websocket',
      state: 'disconnected',
      error: 'WebSocket reconnect attempts exhausted'
    })

    expect(connectedUrls).toEqual([
      'ws://127.0.0.1:8080/api/events/ws',
      'http://127.0.0.1:8080/api/events/stream'
    ])

    client.reconnect()

    expect(disconnectedUrls).toContain('http://127.0.0.1:8080/api/events/stream')
    expect(disconnectedUrls).toContain('ws://127.0.0.1:8080/api/events/ws')
    expect(connectedUrls.at(-1)).toBe('ws://127.0.0.1:8080/api/events/ws')
  })

  it('loads and persists the replay cursor', () => {
    const connect = vi.fn()
    const queryClient = { invalidateQueries: vi.fn() }
    const storage = {
      getItem: vi.fn().mockReturnValue('41'),
      setItem: vi.fn(),
      removeItem: vi.fn(),
      clear: vi.fn(),
      key: vi.fn(),
      length: 1
    }
    vi.stubGlobal('localStorage', storage)
    let capturedOptions: RealtimeClientOptions | null = null

    initializeRealtime(
      {
        apiBaseUrl: 'http://127.0.0.1:8080',
        apiSecret: 'test-secret',
        sseUrl: 'http://127.0.0.1:8080/api/events/stream',
        webSocketUrl: 'ws://127.0.0.1:8080/api/events/ws',
        realtimeTransport: 'sse'
      },
      queryClient,
      (options) => {
        capturedOptions = options
        return { connect, disconnect: vi.fn(), reconnect: vi.fn() }
      }
    )

    const options = capturedOptions as unknown as SseClientOptions
    expect(options.lastEventId).toBe('41')
    options.onMessage?.({ id: '42', event: 'event', data: '{}' })
    expect(storage.setItem).toHaveBeenCalledWith('makosh.realtime.lastEventId', '42')
  })

  it('reports lagged realtime gaps without advancing the replay cursor', () => {
    const connect = vi.fn()
    const queryClient = { invalidateQueries: vi.fn() }
    const storage = {
      getItem: vi.fn().mockReturnValue('41'),
      setItem: vi.fn(),
      removeItem: vi.fn(),
      clear: vi.fn(),
      key: vi.fn(),
      length: 1
    }
    vi.stubGlobal('localStorage', storage)
    const onLaggedObserved = vi.fn()
    let capturedOptions: RealtimeClientOptions | null = null

    initializeRealtime(
      {
        apiBaseUrl: 'http://127.0.0.1:8080',
        apiSecret: 'test-secret',
        sseUrl: 'http://127.0.0.1:8080/api/events/stream',
        webSocketUrl: 'ws://127.0.0.1:8080/api/events/ws',
        realtimeTransport: 'sse'
      },
      queryClient,
      {
        onLaggedObserved,
        createClient: (options) => {
          capturedOptions = options
          return { connect, disconnect: vi.fn(), reconnect: vi.fn() }
        }
      }
    )

    const options = capturedOptions as unknown as SseClientOptions
    options.onMessage?.({ id: '41', event: 'lagged', data: JSON.stringify({ skipped: 3 }) })

    expect(onLaggedObserved).toHaveBeenCalledWith(3)
    expect(storage.setItem).not.toHaveBeenCalled()
  })

  it('does not rewind the persisted replay cursor when an older event arrives', () => {
    const connect = vi.fn()
    const queryClient = { invalidateQueries: vi.fn() }
    const storage = {
      getItem: vi.fn().mockReturnValue('50'),
      setItem: vi.fn(),
      removeItem: vi.fn(),
      clear: vi.fn(),
      key: vi.fn(),
      length: 1
    }
    vi.stubGlobal('localStorage', storage)
    let capturedOptions: RealtimeClientOptions | null = null

    initializeRealtime(
      {
        apiBaseUrl: 'http://127.0.0.1:8080',
        apiSecret: 'test-secret',
        sseUrl: 'http://127.0.0.1:8080/api/events/stream',
        webSocketUrl: 'ws://127.0.0.1:8080/api/events/ws',
        realtimeTransport: 'sse'
      },
      queryClient,
      (options) => {
        capturedOptions = options
        return { connect, disconnect: vi.fn(), reconnect: vi.fn() }
      }
    )

    const options = capturedOptions as unknown as SseClientOptions
    options.onMessage?.({ id: '49', event: 'event', data: '{}' })
    expect(storage.setItem).not.toHaveBeenCalled()

    options.onMessage?.({ id: '51', event: 'event', data: '{}' })
    expect(storage.setItem).toHaveBeenCalledOnce()
    expect(storage.setItem).toHaveBeenCalledWith('makosh.realtime.lastEventId', '51')
  })

  it('invalidates broad communication and telegram queries when realtime reports a replay gap', () => {
    const queryClient = { invalidateQueries: vi.fn() }

    handleRealtimeEvent(
      {
        id: '52',
        event: 'lagged',
        data: JSON.stringify({ skipped: 4 })
      },
      queryClient
    )

    expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
      queryKey: ['communications-list']
    })
    expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
      queryKey: ['communications', 'telegram', 'messages']
    })
    expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
      queryKey: ['integrations', 'telegram', 'runtime']
    })
  })

  it('invalidates targeted mail queries for AI state events', () => {
    const queryClient = { invalidateQueries: vi.fn() }

    handleRealtimeEvent(
      {
        id: '42',
        event: 'event',
        data: JSON.stringify({
          position: 42,
          event: {
            event_type: 'mail.ai_state.changed',
            subject: { id: 'msg:1' }
          }
        })
      },
      queryClient
    )

    expect(queryClient.invalidateQueries).toHaveBeenCalledTimes(3)
    expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
      queryKey: ['communications-message']
    })
    expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
      queryKey: ['communications-ai-state']
    })
    expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
      queryKey: ['communications-list']
    })
  })

  it('patches cached AI state for AI state realtime events', () => {
    const setQueryData = vi.fn((queryKey, updater) =>
      typeof updater === 'function' ? updater(undefined) : updater
    )
    const queryClient = {
      invalidateQueries: vi.fn(),
      getQueriesData: vi.fn().mockReturnValue([]),
      setQueryData
    }

    handleRealtimeEvent(
      {
        id: '52',
        event: 'event',
        data: JSON.stringify({
          event: {
            event_type: 'mail.ai_state.changed',
            payload: {
              message_id: 'msg-1',
              ai_state: 'PROCESSING',
              review_required: false,
              failed: false
            }
          }
        })
      },
      queryClient
    )

    expect(setQueryData).toHaveBeenCalledWith(
      ['communications-ai-state', 'msg-1'],
      expect.any(Function)
    )
    expect(setQueryData.mock.results[0]?.value).toMatchObject({
      message_id: 'msg-1',
      ai_state: 'PROCESSING',
      review_reason: null,
      last_error: null
    })
  })

  it('invalidates saved-search queries for saved search events', () => {
    const queryClient = { invalidateQueries: vi.fn() }

    handleRealtimeEvent(
      {
        id: '43',
        event: 'event',
        data: JSON.stringify({
          event: {
            event_type: 'mail.saved_search.updated'
          }
        })
      },
      queryClient
    )

    expect(queryClient.invalidateQueries).toHaveBeenCalledOnce()
    expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
      queryKey: ['communications-saved-searches']
    })
  })

  it('invalidates only sync status queries for mail sync progress events', () => {
    const queryClient = { invalidateQueries: vi.fn() }

    handleRealtimeEvent(
      {
        id: '44',
        event: 'event',
        data: JSON.stringify({
          event: {
            event_type: 'mail.sync.progress'
          }
        })
      },
      queryClient
    )

    expect(queryClient.invalidateQueries).toHaveBeenCalledOnce()
    expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
      queryKey: ['communications', 'mail', 'sync-statuses']
    })
  })

  it('invalidates targeted message queries for mail message action e
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `frontend/src/platform/bootstrap/realtime.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/platform/bootstrap/realtime.ts`
- Size bytes / Размер в байтах: `17930`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```typescript
import { SseClient, WebSocketClient } from '../sse'
import type {
	SseClientOptions,
	SseMessageEvent,
	SseStatusEvent,
	WebSocketClientOptions,
	WebSocketStatusEvent
} from '../sse'
import type { FrontendConfig } from '../config/env'
import { applyMailRealtimePatch } from '../../domains/communications/queries/realtimeMailPatches'
import { applyWhatsAppRealtimePatch } from '../../domains/communications/queries/realtimeWhatsAppPatches'
import { applyTelegramParticipantRealtimePatch } from '../../domains/communications/queries/realtimeTelegramParticipantPatches'
import { applyTelegramRealtimePatch } from '../../domains/communications/queries/realtimeTelegramPatches'
import { applyTelegramCommandRealtimePatch } from '../../integrations/telegram/queries/realtimeTelegramCommandPatches'
import { applyWhatsAppRuntimeRealtimePatch } from '../../integrations/whatsapp/queries/realtimeWhatsAppRuntimePatches'
import { zoomQueryKeys } from '../../integrations/zoom/queries/zoomQueryKeys'

export type RealtimeClient = {
	connect: () => void
	disconnect: () => void
	reconnect: () => void
}

export type RealtimeQueryClient = {
	invalidateQueries: (filters: { queryKey: readonly unknown[] }) => unknown
	getQueriesData?: <TData>(filters: { queryKey: readonly unknown[] }) => Array<[
		readonly unknown[],
		TData | undefined
	]>
	setQueryData?: <TData>(
		queryKey: readonly unknown[],
		updater: TData | ((data: TData | undefined) => TData | undefined)
	) => unknown
}

export type RealtimeClientOptions = SseClientOptions | WebSocketClientOptions
export type RealtimeClientFactory = (options: RealtimeClientOptions) => RealtimeClient
export type RealtimeStatusHandler = (status: SseStatusEvent | WebSocketStatusEvent) => void

export type RealtimeBootstrapOptions = {
	createClient?: RealtimeClientFactory
	onEventObserved?: (eventId: string) => void
	onLaggedObserved?: (skipped: number) => void
	onStatus?: RealtimeStatusHandler
}

const REALTIME_CURSOR_STORAGE_KEY = 'makosh.realtime.lastEventId'

const REALTIME_QUERY_KEYS: readonly (readonly unknown[])[] = [
	['communications-list'],
	['communications-state-counts'],
	['communications-drafts'],
	['communications-outbox'],
	['communications-threads'],
	['communications-message'],
	['communications-ai-state'],
	['communications-saved-searches'],
	['communications-folders'],
	['communications-folder-messages'],
	['communications-attachment-search']
]

const MAIL_RUNTIME_QUERY_KEYS: readonly (readonly unknown[])[] = [
	['communications', 'mail', 'sync-statuses'],
	['communications', 'mail', 'mailbox-health']
]

const TELEGRAM_QUERY_KEYS: readonly (readonly unknown[])[] = [
	['integrations', 'telegram', 'capabilities'],
	['integrations', 'telegram', 'accounts'],
	['communications', 'telegram', 'chats'],
	['communications', 'telegram', 'folders'],
	['communications', 'telegram', 'messages'],
	['integrations', 'telegram', 'runtime'],
	['communications', 'telegram', 'calls']
]

const WHATSAPP_QUERY_KEYS: readonly (readonly unknown[])[] = [
	['integrations', 'whatsapp', 'capabilities'],
	['integrations', 'whatsapp', 'account-capabilities'],
	['integrations', 'whatsapp', 'sessions'],
	['integrations', 'whatsapp', 'runtime', 'status'],
	['integrations', 'whatsapp', 'runtime', 'health'],
	['integrations', 'whatsapp', 'commands'],
	['integrations', 'whatsapp', 'runtime', 'sync-chats'],
	['integrations', 'whatsapp', 'runtime', 'sync-history'],
	['integrations', 'whatsapp', 'runtime', 'sync-members'],
	['integrations', 'whatsapp', 'runtime', 'sync-statuses'],
	['integrations', 'whatsapp', 'runtime', 'sync-presence'],
	['integrations', 'whatsapp', 'runtime', 'sync-calls'],
	['integrations', 'whatsapp', 'runtime', 'sync-contacts'],
	['integrations', 'whatsapp', 'runtime', 'sync-media'],
	['communications', 'whatsapp', 'conversations'],
	['communications', 'whatsapp', 'conversation-detail'],
	['communications', 'whatsapp', 'chat-members'],
	['communications', 'whatsapp', 'messages']
]

const ZOOM_QUERY_KEYS: readonly (readonly unknown[])[] = [
	zoomQueryKeys.accounts,
	zoomQueryKeys.capabilities,
	zoomQueryKeys.runtimeStatus,
	zoomQueryKeys.webhookSubscriptions,
	zoomQueryKeys.providerCalls,
	zoomQueryKeys.callTranscript,
	zoomQueryKeys.recordingImports,
	zoomQueryKeys.auditEvents
]

const SIGNAL_HUB_QUERY_KEYS: readonly (readonly unknown[])[] = [
	['signal-hub']
]

export function initializeRealtime(
	config: FrontendConfig,
	queryClient: RealtimeQueryClient,
	options: RealtimeClientFactory | RealtimeBootstrapOptions = {}
): RealtimeClient {
	const bootstrapOptions = normalizeRealtimeBootstrapOptions(options)
	const createClient: RealtimeClientFactory =
		bootstrapOptions.createClient ??
		((clientOptions) =>
			adaptRealtimeClient(
				isWebSocketClientOptions(clientOptions)
					? new WebSocketClient(clientOptions)
					: new SseClient(clientOptions)
			))

	const clientOptions = realtimeClientOptions(
		config,
		queryClient,
		bootstrapOptions.onEventObserved,
		bootstrapOptions.onLaggedObserved,
		bootstrapOptions.onStatus
	)
	const createSseClient = (): RealtimeClient => createClient(clientOptions.sse)

	if (config.realtimeTransport !== 'websocket') {
		const client = createSseClient()
		client.connect()
		return {
			connect: () => client.connect(),
			disconnect: () => client.disconnect(),
			reconnect: () => {
				client.disconnect()
				client.connect()
			}
		}
	}

	let sseFallbackClient: RealtimeClient | null = null
	let disconnected = false
	let reconnecting = false
	const webSocketClient = createClient({
		...clientOptions.webSocket,
		onStatus: (status: WebSocketStatusEvent) => {
			bootstrapOptions.onStatus?.(status)
			if (
				status.state === 'disconnected' &&
				!disconnected &&
				!reconnecting &&
				!sseFallbackClient
			) {
				sseFallbackClient = createSseClient()
				sseFallbackClient.connect()
			}
		}
	})
	webSocketClient.connect()

	return {
		connect: () => {
			disconnected = false
			if (sseFallbackClient) {
				sseFallbackClient.connect()
				return
			}
			webSocketClient.connect()
		},
		disconnect: () => {
			disconnected = true
			webSocketClient.disconnect()
			sseFallbackClient?.disconnect()
		},
		reconnect: () => {
			reconnecting = true
			disconnected = true
			sseFallbackClient?.disconnect()
			sseFallbackClient = null
			webSocketClient.disconnect()
			disconnected = false
			reconnecting = false
			webSocketClient.connect()
		}
	}
}

function realtimeClientOptions(
	config: FrontendConfig,
	queryClient: RealtimeQueryClient,
	onEventObserved?: (eventId: string) => void,
	onLaggedObserved?: (skipped: number) => void,
	onStatus?: RealtimeStatusHandler
): { sse: SseClientOptions; webSocket: WebSocketClientOptions } {
	const common = {
		secret: config.apiSecret,
		lastEventId: readRealtimeCursor(),
		onMessage: (event: SseMessageEvent) => {
			if (event.event === 'lagged') {
				onLaggedObserved?.(laggedSkippedCount(event.data))
				handleRealtimeEvent(event, queryClient)
				return
			}

			persistRealtimeCursor(event.id)
			onEventObserved?.(event.id)
			handleRealtimeEvent(event, queryClient)
		}
	}

	return {
		sse: {
			...common,
			url: config.sseUrl,
			longPollUrl: `${config.apiBaseUrl}/api/v1/events`,
			onError: (error) => {
				console.warn('[Realtime] SSE stream unavailable', error)
			},
			onStatus
		},
		webSocket: {
			...common,
			url: config.webSocketUrl,
			onError: (error) => {
				console.warn('[Realtime] WebSocket stream unavailable', error)
			},
			onStatus
		}
	}
}

function normalizeRealtimeBootstrapOptions(
	options: RealtimeClientFactory | RealtimeBootstrapOptions
): RealtimeBootstrapOptions {
	if (typeof options === 'function') {
		return { createClient: options }
	}

	return options
}

function isWebSocketClientOptions(
	options: RealtimeClientOptions
): options is WebSocketClientOptions {
	return options.url.includes('/api/events/ws')
}

function adaptRealtimeClient(client: { connect: () => void; disconnect: () => void }): RealtimeClient {
	return {
		connect: () => client.connect(),
		disconnect: () => client.disconnect(),
		reconnect: () => {
			client.disconnect()
			client.connect()
		}
	}
}

export function handleRealtimeEvent(
	event: SseMessageEvent,
	queryClient: RealtimeQueryClient
): void {
	if (event.event === 'heartbeat') return
	if (event.event === 'error') return
	if (event.event === 'lagged') {
		for (const queryKey of laggedRealtimeQueryKeys()) {
			void queryClient.invalidateQueries({ queryKey })
		}
		return
	}

	applyMailRealtimePatch(event.data, queryClient)
	applyWhatsAppRealtimePatch(event.data, queryClient)
	applyWhatsAppRuntimeRealtimePatch(event.data, queryClient)
	applyTelegramRealtimePatch(event.data, queryClient)
	applyTelegramParticipantRealtimePatch(event.data, queryClient)
	applyTelegramCommandRealtimePatch(event.data, queryClient)

	for (const queryKey of queryKeysForRealtimeEvent(event)) {
		void queryClient.invalidateQueries({ queryKey })
	}
}

function laggedRealtimeQueryKeys(): readonly (readonly unknown[])[] {
	return [
		...REALTIME_QUERY_KEYS,
		...MAIL_RUNTIME_QUERY_KEYS,
		...TELEGRAM_QUERY_KEYS,
		...WHATSAPP_QUERY_KEYS,
		...ZOOM_QUERY_KEYS,
		...SIGNAL_HUB_QUERY_KEYS
	]
}

function queryKeysForRealtimeEvent(event: SseMessageEvent): readonly (readonly unknown[])[] {
	const eventType = canonicalEventType(event.data)
	if (!eventType) return REALTIME_QUERY_KEYS

	if (eventType.startsWith('signal.')) {
		return SIGNAL_HUB_QUERY_KEYS
	}

	if (eventType === 'mail.ai_state.changed') {
		return [['communications-ai-state'], ['communications-message'], ['communications-list']]
	}
	if (eventType === 'mail.read_receipt.recorded') {
		return [['communications-outbox'], ['communications-message'], ['communications-list']]
	}
	if (eventType.startsWith('mail.outbox.')) {
		return [['communications-outbox'], ['communications-list']]
	}
	if (eventType.startsWith('mail.sync.')) {
		return [['communications', 'mail', 'sync-statuses']]
	}
	if (eventType.startsWith('mail.message.')) {
		return [
			['communications-message'],
			['communications-list'],
			['communications-state-counts'],
			['communications-threads'],
			['communications-saved-searches'],
			['communications-folders'],
			['communications-folder-messages']
		]
	}
	if (eventType.startsWith('mail.draft.')) {
		return [['communications-drafts']]
	}
	if (eventType.startsWith('mail.saved_search.')) {
		return [['communications-saved-searches']]
	}
	if (eventType.startsWith('mail.folder_message.')) {
		return [
			['communications-folders'],
			['communications-folder-messages'],
			['communications-list']
		]
	}
	if (eventType.startsWith('mail.folder.')) {
		return [['communications-folders'], ['communications-folder-messages']]
	}
	if (eventType.startsWith('telegram.sync.')) {
		return [['communications', 'telegram', 'chats'], ['communications', 'telegram', 'messages'], ['integrations', 'telegram', 'runtime']]
	}
	if (eventType.startsWith('telegram.message.')) {
		return [['communications', 'telegram', 'messages'], ['communications', 'telegram', 'chats']]
	}
	if (eventType.startsWith('telegram.typing.')) {
		return [['communications', 'telegram', 'chats'], ['integrations', 'telegram', 'runtime']]
	}
	if (eventType.startsWith('telegram.topic.')) {
		return [['communications', 'telegram', 'topics'], ['communications', 'telegram', 'topic-search'], ['communications', 'telegram', 'topic-messages']]
	}
	if (eventType.startsWith('telegram.participant.')) {
		return [['communications', 'telegram', 'chat-members'], ['communications', 'telegram', 'chats']]
	}
	if (eventType.startsWith('telegram.folders.')) {
		return [['communications', 'telegram', 'folders'], ['communications', 'telegram', 'chats']]
	}
	if (eventType.startsWith('telegram.media.upload.')) {
		return [['integrations', 'telegram', 'commands'], ['integrations', 'telegram', 'runtime']]
	}
	if (eventType.startsWith('telegram.media.download.')) {
		return [['communications', 'telegram', 'messages'], ['communications', 'telegram', 'search', 'media']]
	}
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `frontend/src/platform/bootstrap/realtimeCachePatches.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/platform/bootstrap/realtimeCachePatches.test.ts`
- Size bytes / Размер в байтах: `7969`
- Included characters / Включено символов: `7969`
- Truncated / Обрезано: `no`

```typescript
import { describe, expect, it, vi } from 'vitest'
import { handleRealtimeEvent } from './realtime'

describe('realtime cache patch handling', () => {
  it('patches cached folder-message lists for moved message realtime events', () => {
    const sourceKey = ['communications-folder-messages', 'folder-old']
    const targetKey = ['communications-folder-messages', 'folder-new']
    const movedMessage = {
      folder_id: 'folder-new',
      message_id: 'msg-1',
      account_id: 'account-1',
      subject: 'Project update',
      sender: 'sender@example.com',
      occurred_at: '2026-06-15T09:00:00Z',
      projected_at: '2026-06-15T09:01:00Z',
      workflow_state: 'new',
      local_state: 'active',
      added_at: '2026-06-15T10:00:00Z',
      attachment_count: 0
    }
    const sourceData = {
      pages: [
        {
          items: [{ ...movedMessage, folder_id: 'folder-old' }],
          next_cursor: null,
          has_more: false
        }
      ],
      pageParams: [null]
    }
    const targetData = {
      pages: [
        {
          items: [],
          next_cursor: null,
          has_more: false
        }
      ],
      pageParams: [null]
    }
    const setQueryData = vi.fn()
    const queryClient = {
      invalidateQueries: vi.fn(),
      getQueriesData: vi.fn().mockReturnValue([
        [sourceKey, sourceData],
        [targetKey, targetData]
      ]),
      setQueryData
    }

    handleRealtimeEvent(
      {
        id: '58',
        event: 'event',
        data: JSON.stringify({
          event: {
            event_type: 'mail.folder_message.moved',
            payload: {
              operation: 'move',
              folder_id: 'folder-new',
              message_id: 'msg-1',
              message: movedMessage
            }
          }
        })
      },
      queryClient
    )

    expect(setQueryData).toHaveBeenCalledWith(sourceKey, {
      ...sourceData,
      pages: [{ ...sourceData.pages[0], items: [] }]
    })
    expect(setQueryData).toHaveBeenCalledWith(targetKey, {
      ...targetData,
      pages: [{ ...targetData.pages[0], items: [movedMessage] }]
    })
  })

  it('patches cached saved-search lists for saved search realtime events', () => {
    const savedSearchKey = ['communications-saved-searches', false, undefined]
    const smartFolderKey = ['communications-saved-searches', true, undefined]
    const savedSearch = {
      saved_search_id: 'search-1',
      name: 'Invoices',
      description: null,
      account_id: null,
      query: 'invoice',
      workflow_state: null,
      local_state: 'active',
      channel_kind: 'email',
      is_smart_folder: false,
      sort_order: 1000,
      message_count: 2,
      created_at: '2026-06-15T10:00:00Z',
      updated_at: '2026-06-15T10:00:00Z'
    }
    const smartFolder = {
      ...savedSearch,
      saved_search_id: 'smart-1',
      name: 'Needs Action',
      query: 'state:needs_action',
      is_smart_folder: true
    }
    const savedSearchData = {
      pages: [{ items: [savedSearch], next_cursor: null, has_more: false }],
      pageParams: [null]
    }
    const smartFolderData = {
      pages: [{ items: [smartFolder], next_cursor: null, has_more: false }],
      pageParams: [null]
    }
    const setQueryData = vi.fn()
    const queryClient = {
      invalidateQueries: vi.fn(),
      getQueriesData: vi.fn().mockReturnValue([
        [savedSearchKey, savedSearchData],
        [smartFolderKey, smartFolderData]
      ]),
      setQueryData
    }

    handleRealtimeEvent(
      {
        id: '56',
        event: 'event',
        data: JSON.stringify({
          event: {
            event_type: 'mail.saved_search.updated',
            payload: {
              ...savedSearch,
              name: 'Paid invoices',
              message_count: 3,
              updated_at: '2026-06-15T10:01:00Z'
            }
          }
        })
      },
      queryClient
    )

    expect(setQueryData).toHaveBeenCalledOnce()
    expect(setQueryData).toHaveBeenCalledWith(savedSearchKey, {
      ...savedSearchData,
      pages: [
        {
          ...savedSearchData.pages[0],
          items: [
            {
              ...savedSearch,
              name: 'Paid invoices',
              message_count: 3,
              updated_at: '2026-06-15T10:01:00Z'
            }
          ]
        }
      ]
    })
  })

  it('patches cached sync statuses for sync progress realtime events', () => {
    const syncKey = ['communications', 'mail', 'sync-statuses']
    const statuses = [
      {
        account_id: 'account-1',
        status: 'running',
        phase: 'fetch',
        progress_mode: 'determinate',
        progress_percent: 10,
        processed_messages: 5,
        estimated_total_messages: 50,
        current_batch_size: 10,
        last_started_at: '2026-06-15T10:00:00Z',
        last_completed_at: null,
        next_run_at: null,
        last_error_code: null,
        last_error_message: null,
        last_fetched_messages: 5,
        last_projected_messages: 4,
        last_upserted_persons: 1,
        last_upserted_organizations: 0
      }
    ]
    const setQueryData = vi.fn((queryKey, updater) =>
      typeof updater === 'function' ? updater(statuses) : updater
    )
    const queryClient = {
      invalidateQueries: vi.fn(),
      getQueriesData: vi.fn().mockReturnValue([[syncKey, statuses]]),
      setQueryData
    }

    handleRealtimeEvent(
      {
        id: '57',
        event: 'event',
        data: JSON.stringify({
          event: {
            event_type: 'mail.sync.progress',
            payload: {
              account_id: 'account-1',
              status: 'running',
              phase: 'project',
              progress_mode: 'determinate',
              progress_percent: 60,
              processed_messages: 30,
              estimated_total_messages: 50,
              current_batch_size: 10,
              fetched_messages: 30,
              projected_messages: 24,
              upserted_persons: 5,
              upserted_organizations: 2,
              next_run_at: null
            }
          }
        })
      },
      queryClient
    )

    expect(setQueryData).toHaveBeenCalledWith(syncKey, expect.any(Array))
    expect(setQueryData.mock.results[0]?.value[0]).toMatchObject({
      account_id: 'account-1',
      phase: 'project',
      progress_percent: 60,
      processed_messages: 30,
      last_fetched_messages: 30,
      last_projected_messages: 24,
      last_upserted_persons: 5,
      last_upserted_organizations: 2
    })
  })

  it('invalidates only draft queries for mail draft events', () => {
    const queryClient = { invalidateQueries: vi.fn() }

    handleRealtimeEvent(
      {
        id: '46',
        event: 'event',
        data: JSON.stringify({
          event: {
            event_type: 'mail.draft.updated'
          }
        })
      },
      queryClient
    )

    expect(queryClient.invalidateQueries).toHaveBeenCalledOnce()
    expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
      queryKey: ['communications-drafts']
    })
  })

  it('falls back to broad invalidation for unknown canonical events', () => {
    const queryClient = { invalidateQueries: vi.fn() }

    handleRealtimeEvent(
      {
        id: '44',
        event: 'event',
        data: JSON.stringify({
          event: {
            event_type: 'mail.unknown.changed'
          }
        })
      },
      queryClient
    )

    expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
      queryKey: ['communications', 'mail', 'sync-statuses']
    })
    expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
      queryKey: ['communications-attachment-search']
    })
  })

  it('ignores heartbeat events', () => {
    const queryClient = { invalidateQueries: vi.fn() }

    handleRealtimeEvent({ id: '', event: 'heartbeat', data: '{}' }, queryClient)

    expect(queryClient.invalidateQueries).not.toHaveBeenCalled()
  })
})
```

### `frontend/src/platform/bootstrap/realtimeMailCachePatches.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/platform/bootstrap/realtimeMailCachePatches.test.ts`
- Size bytes / Размер в байтах: `6459`
- Included characters / Включено символов: `6459`
- Truncated / Обрезано: `no`

```typescript
import { describe, expect, it, vi } from 'vitest'
import { handleRealtimeEvent } from './realtime'

describe('realtime bootstrap mail cache patches', () => {
  it('patches cached outbox metadata for delivery status and read receipt events', () => {
    const outboxKey = ['communications-outbox', undefined, undefined]
    const outboxItems = {
      pages: [
        {
          items: [
            {
              outbox_id: 'outbox-1',
              account_id: 'account-1',
              status: 'sent',
              provider_message_id: 'provider-1',
              last_error: null,
              send_attempts: 1,
              scheduled_send_at: null,
              undo_deadline_at: null,
              sent_at: '2026-06-15T10:00:00Z',
              metadata: {}
            }
          ],
          next_cursor: null,
          has_more: false
        }
      ],
      pageParams: [null]
    }
    const setQueryData = vi.fn((queryKey, updater) =>
      typeof updater === 'function' ? updater(outboxItems) : updater
    )
    const queryClient = {
      invalidateQueries: vi.fn(),
      getQueriesData: vi.fn().mockReturnValue([[outboxKey, outboxItems]]),
      setQueryData
    }

    handleRealtimeEvent(
      {
        id: '50',
        event: 'event',
        data: JSON.stringify({
          event: {
            event_type: 'mail.outbox.delivery_status_changed',
            payload: {
              outbox_id: 'outbox-1',
              delivery_status: 'delivered',
              source_kind: 'provider_runtime',
              recorded_at: '2026-06-15T10:01:00Z'
            }
          }
        })
      },
      queryClient
    )
    const patchedDeliveryItems = setQueryData.mock.results[0]?.value
    expect(patchedDeliveryItems.pages[0].items[0].metadata.delivery_status).toMatchObject({
      delivery_status: 'delivered',
      source_kind: 'provider_runtime',
      recorded_at: '2026-06-15T10:01:00Z'
    })

    handleRealtimeEvent(
      {
        id: '51',
        event: 'event',
        data: JSON.stringify({
          event: {
            event_type: 'mail.read_receipt.recorded',
            payload: {
              outbox_id: 'outbox-1',
              receipt_id: 'receipt-1',
              receipt_kind: 'read',
              read_at: '2026-06-15T10:02:00Z',
              source_kind: 'provider_runtime'
            }
          }
        })
      },
      queryClient
    )

    const patchedReadItems = setQueryData.mock.results[1]?.value
    expect(patchedReadItems.pages[0].items[0].metadata.latest_read_receipt).toMatchObject({
      receipt_id: 'receipt-1',
      receipt_kind: 'read',
      read_at: '2026-06-15T10:02:00Z'
    })
  })

  it('removes cached drafts for draft deleted realtime events', () => {
    const draftsKey = ['communications-drafts', 'account-1']
    const drafts = [
      {
        draft_id: 'draft-1',
        account_id: 'account-1',
        persona_id: null,
        to_recipients: [],
        cc_recipients: [],
        bcc_recipients: [],
        subject: '',
        body_text: '',
        body_html: null,
        in_reply_to: null,
        references: [],
        status: 'draft',
        scheduled_send_at: null,
        send_attempts: 0,
        last_error: null,
        metadata: {},
        created_at: '2026-06-15T10:00:00Z',
        updated_at: '2026-06-15T10:00:00Z'
      },
      {
        draft_id: 'draft-2',
        account_id: 'account-1',
        persona_id: null,
        to_recipients: [],
        cc_recipients: [],
        bcc_recipients: [],
        subject: '',
        body_text: '',
        body_html: null,
        in_reply_to: null,
        references: [],
        status: 'draft',
        scheduled_send_at: null,
        send_attempts: 0,
        last_error: null,
        metadata: {},
        created_at: '2026-06-15T10:00:00Z',
        updated_at: '2026-06-15T10:00:00Z'
      }
    ]
    const setQueryData = vi.fn((queryKey, updater) =>
      typeof updater === 'function' ? updater(drafts) : updater
    )
    const queryClient = {
      invalidateQueries: vi.fn(),
      getQueriesData: vi.fn().mockReturnValue([[draftsKey, drafts]]),
      setQueryData
    }

    handleRealtimeEvent(
      {
        id: '53',
        event: 'event',
        data: JSON.stringify({
          event: {
            event_type: 'mail.draft.deleted',
            payload: {
              draft_id: 'draft-1',
              account_id: 'account-1'
            }
          }
        })
      },
      queryClient
    )

    expect(setQueryData.mock.results[0]?.value).toEqual([drafts[1]])
  })

  it('patches cached folder lists for folder realtime events', () => {
    const foldersKey = ['communications-folders', undefined]
    const folder = {
      folder_id: 'folder-1',
      account_id: null,
      name: 'Projects',
      description: null,
      color: null,
      sort_order: 1000,
      message_count: 3,
      created_at: '2026-06-15T10:00:00Z',
      updated_at: '2026-06-15T10:00:00Z'
    }
    const folderData = {
      pages: [
        {
          items: [folder],
          next_cursor: null,
          has_more: false
        }
      ],
      pageParams: [null]
    }
    const setQueryData = vi.fn((queryKey, updater) =>
      typeof updater === 'function' ? updater(folderData) : updater
    )
    const queryClient = {
      invalidateQueries: vi.fn(),
      getQueriesData: vi.fn().mockReturnValue([[foldersKey, folderData]]),
      setQueryData
    }

    handleRealtimeEvent(
      {
        id: '54',
        event: 'event',
        data: JSON.stringify({
          event: {
            event_type: 'mail.folder.updated',
            payload: {
              ...folder,
              name: 'Clients',
              message_count: 4,
              updated_at: '2026-06-15T10:01:00Z'
            }
          }
        })
      },
      queryClient
    )

    const patchedUpdate = setQueryData.mock.results[0]?.value
    expect(patchedUpdate.pages[0].items[0]).toMatchObject({
      folder_id: 'folder-1',
      name: 'Clients',
      message_count: 4
    })

    handleRealtimeEvent(
      {
        id: '55',
        event: 'event',
        data: JSON.stringify({
          event: {
            event_type: 'mail.folder.deleted',
            payload: folder
          }
        })
      },
      queryClient
    )

    const patchedDelete = setQueryData.mock.results[1]?.value
    expect(patchedDelete.pages[0].items).toEqual([])
  })
})
```

### `frontend/src/platform/bootstrap/realtimeSignalHubInvalidation.test.ts`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/platform/bootstrap/realtimeSignalHubInvalidation.test.ts`
- Size bytes / Размер в байтах: `1493`
- Included characters / Включено символов: `1493`
- Truncated / Обрезано: `no`

```typescript
import { describe, expect, it, vi } from 'vitest'
import { handleRealtimeEvent } from './realtime'

describe('Signal Hub realtime invalidation', () => {
  it('invalidates Signal Hub queries for every supported signal event family', () => {
    const signalEventTypes = [
      'signal.raw.telegram.message.observed',
      'signal.accepted.telegram.message',
      'signal.rejected.mail.message',
      'signal.muted.system.runtime',
      'signal.paused.ai.task',
      'signal.resumed.telegram.runtime',
      'signal.replayed.whatsapp.message'
    ] as const

    for (const eventType of signalEventTypes) {
      const queryClient = { invalidateQueries: vi.fn() }

      handleRealtimeEvent(
        {
          id: '42',
          event: 'event',
          data: JSON.stringify({
            event: {
              event_type: eventType
            }
          })
        },
        queryClient
      )

      expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
        queryKey: ['signal-hub']
      })
      expect(queryClient.invalidateQueries).toHaveBeenCalledTimes(1)
    }
  })

  it('keeps Signal Hub invalidation on lagged stream recovery', () => {
    const queryClient = { invalidateQueries: vi.fn() }

    handleRealtimeEvent(
      {
        id: '42',
        event: 'lagged',
        data: JSON.stringify({ skipped: 7 })
      },
      queryClient
    )

    expect(queryClient.invalidateQueries).toHaveBeenCalledWith({
      queryKey: ['signal-hub']
    })
  })
})
```
