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
const personsProtoRoot = join(repoRoot, 'backend', 'src', 'persons-api', 'proto')
const tasksProtoRoot = join(repoRoot, 'backend', 'src', 'tasks-command-api', 'proto')
const knowledgeProtoRoot = join(repoRoot, 'backend', 'src', 'knowledge-command-api', 'proto')
const calendarProtoRoot = join(repoRoot, 'backend', 'src', 'calendar-api', 'proto')
const organizationsProtoRoot = join(repoRoot, 'backend', 'src', 'organizations-api', 'proto')
const documentsProtoRoot = join(repoRoot, 'backend', 'src', 'documents-api', 'proto')
const relationshipsProtoRoot = join(repoRoot, 'backend', 'src', 'relationships-api', 'proto')
const projectsProtoRoot = join(repoRoot, 'backend', 'src', 'projects-api', 'proto')
const obligationsProtoRoot = join(repoRoot, 'backend', 'src', 'obligations-api', 'proto')
const decisionsProtoRoot = join(repoRoot, 'backend', 'src', 'decisions-api', 'proto')
const searchProtoRoot = join(repoRoot, 'backend', 'src', 'search-api', 'proto')
const timelineProtoRoot = join(repoRoot, 'backend', 'src', 'timeline-api', 'proto')
const graphProtoRoot = join(repoRoot, 'backend', 'src', 'graph-api', 'proto')
const memoryProtoRoot = join(repoRoot, 'backend', 'src', 'memory-api', 'proto')
const consistencyProtoRoot = join(repoRoot, 'backend', 'src', 'consistency-api', 'proto')
const riskProtoRoot = join(repoRoot, 'backend', 'src', 'risk-api', 'proto')
const zoomProtoRoot = join(repoRoot, 'backend', 'src', 'zoom-api', 'proto')
const telemostProtoRoot = join(repoRoot, 'backend', 'src', 'telemost-api', 'proto')
const reviewPersonMatchCandidateProtoRoot = join(
  repoRoot,
  'backend',
  'src',
  'review-person-match-candidate-api',
  'proto'
)
const reviewAttentionProtoRoot = join(repoRoot, 'backend', 'src', 'review-attention-api', 'proto')
const reviewTaskCandidateProtoRoot = join(
  repoRoot,
  'backend',
  'src',
  'review-task-candidate-api',
  'proto'
)
const reviewObligationCandidateProtoRoot = join(
  repoRoot,
  'backend',
  'src',
  'review-obligation-candidate-api',
  'proto'
)
const reviewNoteCandidateProtoRoot = join(
  repoRoot,
  'backend',
  'src',
  'review-note-candidate-api',
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
  join(personsProtoRoot, 'makosh', 'persons', 'v1', 'persons.proto'),
  join(tasksProtoRoot, 'makosh', 'tasks', 'client', 'v1', 'tasks.proto'),
  join(knowledgeProtoRoot, 'makosh', 'knowledge', 'client', 'v1', 'knowledge.proto'),
  join(calendarProtoRoot, 'makosh', 'calendar', 'client', 'v1', 'calendar.proto'),
  join(organizationsProtoRoot, 'makosh', 'organizations', 'client', 'v1', 'organizations.proto'),
  join(documentsProtoRoot, 'makosh', 'documents', 'client', 'v1', 'documents.proto'),
  join(relationshipsProtoRoot, 'makosh', 'relationships', 'client', 'v1', 'relationships.proto'),
  join(projectsProtoRoot, 'makosh', 'projects', 'client', 'v1', 'projects.proto'),
  join(obligationsProtoRoot, 'makosh', 'obligations', 'client', 'v1', 'obligations.proto'),
  join(decisionsProtoRoot, 'makosh', 'decisions', 'client', 'v1', 'decisions.proto'),
  join(searchProtoRoot, 'makosh', 'search', 'v1', 'search.proto'),
  join(timelineProtoRoot, 'makosh', 'timeline', 'v1', 'timeline.proto'),
  join(graphProtoRoot, 'makosh', 'graph', 'v1', 'graph.proto'),
  join(memoryProtoRoot, 'makosh', 'memory', 'v1', 'memory.proto'),
  join(consistencyProtoRoot, 'makosh', 'consistency', 'v1', 'consistency.proto'),
  join(riskProtoRoot, 'makosh', 'risk', 'v1', 'risk.proto'),
  join(zoomProtoRoot, 'makosh', 'zoom', 'v1', 'zoom.proto'),
  join(telemostProtoRoot, 'makosh', 'telemost', 'v1', 'telemost.proto'),
  join(reviewAttentionProtoRoot, 'makosh', 'review', 'attention', 'client', 'v1', 'client.proto'),
  join(reviewTaskCandidateProtoRoot, 'makosh', 'review', 'task_candidate', 'v1', 'task_candidate.proto'),
  join(reviewObligationCandidateProtoRoot, 'makosh', 'review', 'obligation_candidate', 'v1', 'obligation_candidate.proto'),
  join(reviewNoteCandidateProtoRoot, 'makosh', 'review', 'note_candidate', 'v1', 'note_candidate.proto'),
  join(
    reviewPersonMatchCandidateProtoRoot,
    'makosh',
    'review',
    'person_match_candidate',
    'v1',
    'person_match_candidate.proto'
  ),
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
    `-I${personsProtoRoot}`,
    `-I${tasksProtoRoot}`,
    `-I${knowledgeProtoRoot}`,
    `-I${calendarProtoRoot}`,
    `-I${organizationsProtoRoot}`,
    `-I${documentsProtoRoot}`,
    `-I${relationshipsProtoRoot}`,
    `-I${projectsProtoRoot}`,
    `-I${obligationsProtoRoot}`,
    `-I${decisionsProtoRoot}`,
    `-I${searchProtoRoot}`,
    `-I${timelineProtoRoot}`,
    `-I${graphProtoRoot}`,
    `-I${memoryProtoRoot}`,
    `-I${consistencyProtoRoot}`,
    `-I${riskProtoRoot}`,
    `-I${zoomProtoRoot}`,
    `-I${telemostProtoRoot}`,
    `-I${reviewAttentionProtoRoot}`,
    `-I${reviewTaskCandidateProtoRoot}`,
    `-I${reviewObligationCandidateProtoRoot}`,
    `-I${reviewNoteCandidateProtoRoot}`,
    `-I${reviewPersonMatchCandidateProtoRoot}`,
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
