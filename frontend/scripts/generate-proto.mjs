import { mkdirSync } from 'node:fs'
import { dirname, join, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'
import { spawnSync } from 'node:child_process'

const __dirname = dirname(fileURLToPath(import.meta.url))
const frontendRoot = resolve(__dirname, '..')
const repoRoot = resolve(frontendRoot, '..')
const protoRoot = join(repoRoot, 'contracts', 'proto')
const gatewayProtoRoot = join(repoRoot, 'backend', 'src', 'api', 'gateway', 'contracts', 'proto')
const communicationsQueryProtoRoot = join(repoRoot, 'backend', 'src', 'communications-api', 'proto')
const communicationsContentProtoRoot = join(
  repoRoot,
  'backend',
  'src',
  'communications-content-api',
  'proto'
)
const communicationsSavedSearchProtoRoot = join(
  repoRoot,
  'backend',
  'src',
  'communications-saved-query-api',
  'proto'
)
const communicationsSenderInsightsProtoRoot = join(
  repoRoot,
  'backend',
  'src',
  'communications-sender-insights-api',
  'proto'
)
const communicationsExportProtoRoot = join(
  repoRoot,
  'backend',
  'src',
  'communications-export-api',
  'proto'
)
const attachmentPreviewProtoRoot = join(
  repoRoot,
  'backend',
  'src',
  'attachment-preview-api',
  'proto'
)
const attachmentPreviewEvidenceReplayProtoRoot = join(
  repoRoot,
  'backend',
  'src',
  'attachment-preview-evidence-replay-api',
  'proto'
)
const callTranscriptionProtoRoot = join(
  repoRoot,
  'backend',
  'src',
  'call-transcription-api',
  'proto'
)
const speechTranscriptProtoRoot = join(
  repoRoot,
  'backend',
  'src',
  'speech-transcript-artifact',
  'proto'
)
const mailProtoRoot = join(repoRoot, 'backend', 'src', 'mail-api', 'proto')
const mailContactsSyncProtoRoot = join(
  repoRoot,
  'backend',
  'src',
  'mail-contacts-sync-api',
  'proto'
)
const telegramProtoRoot = join(repoRoot, 'backend', 'src', 'telegram-api', 'proto')
const telegramAutomationProtoRoot = join(repoRoot, 'backend', 'src', 'telegram-automation-api', 'proto')
const whatsappProtoRoot = join(repoRoot, 'backend', 'src', 'whatsapp-api', 'proto')
const zulipProtoRoot = join(repoRoot, 'backend', 'src', 'zulip-api', 'proto')
const outputDir = join(frontendRoot, 'src', 'gen')
const pluginPath = join(frontendRoot, 'node_modules', '.bin', 'protoc-gen-es')
const protoFiles = [
  join(protoRoot, 'makosh', 'common', 'v1', 'common.proto'),
  join(protoRoot, 'makosh', 'events', 'v1', 'event_envelope.proto'),
  join(protoRoot, 'makosh', 'signal_hub', 'v1', 'signal_hub.proto'),
  join(protoRoot, 'makosh', 'communications', 'v1', 'communications.proto'),
  join(communicationsQueryProtoRoot, 'makosh', 'communications', 'query', 'v1', 'query.proto'),
  join(
    communicationsContentProtoRoot,
    'makosh',
    'communications',
    'content',
    'ticket',
    'v1',
    'ticket.proto'
  ),
  join(
    communicationsContentProtoRoot,
    'makosh',
    'communications',
    'content',
    'read',
    'v1',
    'read.proto'
  ),
  join(
    communicationsSavedSearchProtoRoot,
    'makosh',
    'communications',
    'saved_search',
    'v1',
    'saved_search.proto'
  ),
  join(
    communicationsSenderInsightsProtoRoot,
    'makosh',
    'communications',
    'sender_insights',
    'v1',
    'sender_insights.proto'
  ),
  join(
    communicationsExportProtoRoot,
    'makosh',
    'communications_export',
    'v1',
    'export.proto'
  ),
  join(
    attachmentPreviewProtoRoot,
    'makosh',
    'attachment_preview',
    'v1',
    'preview.proto'
  ),
  join(
    attachmentPreviewProtoRoot,
    'makosh',
    'attachment_preview',
    'read',
    'v1',
    'read.proto'
  ),
  join(
    attachmentPreviewEvidenceReplayProtoRoot,
    'makosh',
    'attachment_preview_evidence_replay',
    'v1',
    'replay.proto'
  ),
  join(
    callTranscriptionProtoRoot,
    'makosh',
    'call_transcription',
    'v1',
    'transcription.proto'
  ),
  join(speechTranscriptProtoRoot, 'makosh', 'speech_transcript', 'v1', 'transcript.proto'),
  join(mailProtoRoot, 'makosh', 'mail', 'v1', 'client.proto'),
  join(mailProtoRoot, 'makosh', 'mail', 'account', 'v1', 'client.proto'),
  join(mailProtoRoot, 'makosh', 'mail', 'account_lifecycle', 'v1', 'client.proto'),
  join(mailProtoRoot, 'makosh', 'mail', 'composition', 'v1', 'client.proto'),
  join(mailProtoRoot, 'makosh', 'mail', 'message_flags', 'v1', 'client.proto'),
  join(mailProtoRoot, 'makosh', 'mail', 'message_location', 'v1', 'client.proto'),
  join(mailProtoRoot, 'makosh', 'mail', 'message_permanent_delete', 'v1', 'client.proto'),
  join(mailProtoRoot, 'makosh', 'mail', 'operational', 'v1', 'client.proto'),
  join(mailProtoRoot, 'makosh', 'mail', 'sync_health', 'v1', 'client.proto'),
  join(mailProtoRoot, 'makosh', 'mail', 'portability', 'v1', 'portability.proto'),
  join(mailContactsSyncProtoRoot, 'makosh', 'mail_contacts_sync', 'v1', 'sync.proto'),
  join(telegramProtoRoot, 'makosh', 'telegram', 'v1', 'client.proto'),
  join(telegramAutomationProtoRoot, 'makosh', 'telegram', 'automation', 'v1', 'automation.proto'),
  join(whatsappProtoRoot, 'makosh', 'whatsapp', 'v1', 'client.proto'),
  join(whatsappProtoRoot, 'makosh', 'whatsapp', 'operational', 'v1', 'client.proto'),
  join(whatsappProtoRoot, 'makosh', 'whatsapp', 'operational', 'realtime', 'v1', 'client.proto'),
  join(zulipProtoRoot, 'makosh', 'zulip', 'account', 'v1', 'client.proto'),
  join(zulipProtoRoot, 'makosh', 'zulip', 'operational', 'v1', 'client.proto'),
  join(zulipProtoRoot, 'makosh', 'zulip', 'operational', 'realtime', 'v1', 'client.proto'),
  join(zulipProtoRoot, 'makosh', 'zulip', 'v1', 'client.proto'),
  join(gatewayProtoRoot, 'makosh', 'gateway', 'v1', 'client_realtime.proto'),
  join(gatewayProtoRoot, 'makosh', 'gateway', 'v1', 'client_system_status_realtime.proto'),
  join(gatewayProtoRoot, 'makosh', 'gateway', 'v1', 'browser_session.proto'),
  join(gatewayProtoRoot, 'makosh', 'gateway', 'v1', 'client_bootstrap.proto'),
  join(gatewayProtoRoot, 'makosh', 'gateway', 'v1', 'owner_vault_provisioning.proto'),
  join(gatewayProtoRoot, 'makosh', 'gateway', 'v1', 'owner_module_settings.proto')
]

mkdirSync(outputDir, { recursive: true })

const result = spawnSync(
  'protoc',
  [
    `-I${protoRoot}`,
    `-I${gatewayProtoRoot}`,
    `-I${communicationsQueryProtoRoot}`,
    `-I${communicationsContentProtoRoot}`,
    `-I${communicationsSavedSearchProtoRoot}`,
    `-I${communicationsSenderInsightsProtoRoot}`,
    `-I${communicationsExportProtoRoot}`,
    `-I${attachmentPreviewProtoRoot}`,
    `-I${attachmentPreviewEvidenceReplayProtoRoot}`,
    `-I${callTranscriptionProtoRoot}`,
    `-I${speechTranscriptProtoRoot}`,
    `-I${mailProtoRoot}`,
    `-I${mailContactsSyncProtoRoot}`,
    `-I${telegramProtoRoot}`,
    `-I${telegramAutomationProtoRoot}`,
    `-I${whatsappProtoRoot}`,
    `-I${zulipProtoRoot}`,
    `--plugin=protoc-gen-es=${pluginPath}`,
    `--es_out=${outputDir}`,
    '--es_opt',
    'target=ts',
    ...protoFiles
  ],
  {
    cwd: frontendRoot,
    stdio: 'inherit'
  }
)

if (result.status !== 0) {
  process.exit(result.status ?? 1)
}
