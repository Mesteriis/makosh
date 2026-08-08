// Historical pre-clean-room Mail workspace orchestration. It is not part of the active client graph.
import type {
  CommunicationMessageActionGroupModel,
  CommunicationMessageActionModel
} from '../components/communicationDomainElements'
import type {
  CommunicationMessageDetailItem,
  CommunicationMessageSummary
} from '../types/communications'
import type { useCommunicationsPageSurface } from './useCommunicationsPageSurface'

type CommunicationsPageSurface = ReturnType<typeof useCommunicationsPageSurface>
type MailActionCapabilities = {
  providerFlagMutationAvailable?: boolean
}

export async function selectMailWorkspaceAction(
  pageSurface: CommunicationsPageSurface,
  actionId: string
): Promise<void> {
  if (pageSurface.store.isMailActionRunning) return

  switch (actionId) {
    case 'reply':
      pageSurface.handleReply()
      return
    case 'reply-all':
      pageSurface.handleReplyAll()
      return
    case 'forward':
      pageSurface.handleForwardMessage()
      return
    case 'ai-reply':
      await pageSurface.handleGenerateAiReply({ tone: 'balanced', language: 'auto' })
      return
    case 'smart-cc':
      await pageSurface.handleReviewRecipients()
      return
    case 'forward-eml':
    case 'export-eml':
      await pageSurface.handleExportMessage('eml')
      return
    case 'mark-read':
      await pageSurface.handleMarkMessageRead()
      return
    case 'mark-unread':
      await pageSurface.handleMarkMessageUnread()
      return
    case 'mark-spam':
      await pageSurface.handleMarkMessageSpam()
      return
    case 'mark-not-spam':
      await pageSurface.handleMarkMessageNotSpam()
      return
    case 'mute':
      await pageSurface.handleMute()
      return
    case 'pin':
      await pageSurface.handleTogglePin()
      return
    case 'important':
      await pageSurface.handleToggleImportant()
      return
    case 'star':
      await pageSurface.handleToggleStar()
      return
    case 'analyze':
      await pageSurface.handleAnalyze()
      return
    case 'update-ai-state':
      await pageSurface.handleRetryAi()
      return
    case 'translate':
      await pageSurface.handleTranslate()
      return
    case 'extract-tasks':
    case 'create-task':
      await pageSurface.handleCreateTask()
      return
    case 'extract-notes':
    case 'create-document':
      await pageSurface.handleCreateNote()
      return
    case 'auth-check':
    case 'spf-dkim':
      await pageSurface.handleReviewSecurity()
      return
    case 'export-md':
      await pageSurface.handleExportMessage('md')
      return
    case 'export-json':
      await pageSurface.handleExportMessage('json')
      return
    case 'delete-provider':
      await pageSurface.handleDeleteFromProvider()
      return
    default:
      pageSurface.notifyMailActionError(`Unsupported mail action: ${actionId}`)
  }
}

export function mailActionGroups(
  source: CommunicationMessageDetailItem | CommunicationMessageSummary,
  capabilities: MailActionCapabilities = {}
): CommunicationMessageActionGroupModel[] {
  const providerFlagMutationAvailable = capabilities.providerFlagMutationAvailable ?? true

  return [
    {
      id: 'response',
      title: 'Response',
      actions: [
        mailAction('reply', 'Reply', 'Open a reply draft for this message.', 'tabler:corner-up-left'),
        mailAction('reply-all', 'Reply all', 'Open a reply-all draft for all visible recipients.', 'tabler:corner-up-left-double'),
        mailAction('forward', 'Forward', 'Open a forward draft for this message.', 'tabler:send'),
        mailAction('ai-reply', 'AI reply', 'Generate an evidence-backed draft reply.', 'tabler:sparkles', 'accent', 'communication.mail.ai_reply'),
        mailAction('smart-cc', 'Review recipients', 'Review recipients before replying.', 'tabler:users', undefined, 'communication.mail.recipients.review'),
        mailAction('forward-eml', 'Export EML', 'Export this message as an EML file.', 'tabler:file-export', undefined, 'communication.mail.export.eml')
      ]
    },
    {
      id: 'state',
      title: 'State',
      actions: [
        mailAction('mark-read', 'Mark read', 'Mark this message as read.', 'tabler:mail-opened', 'info', 'communication.mail.read'),
        mailAction('mark-unread', 'Mark unread', 'Mark this message as unread.', 'tabler:mail', 'info', 'communication.mail.unread'),
        spamStateAction(source),
        mailAction('mute', 'Mute', 'Mute this message thread.', 'tabler:volume-off', undefined, 'communication.mail.mute')
      ]
    },
    {
      id: 'organization',
      title: 'Organization',
      actions: [
        mailAction('pin', 'Pin', 'Toggle pin for this message.', 'tabler:pin', undefined, 'communication.mail.pin'),
        mailAction('important', importantLabel(source), providerFlagActionDescription('Toggle the important marker for this message.', providerFlagMutationAvailable), 'tabler:alert-circle', 'warning', 'communication.mail.important'),
        mailAction('star', starLabel(source), providerFlagActionDescription('Toggle the provider-synchronized star for this message.', providerFlagMutationAvailable), 'tabler:star', 'warning', 'communication.mail.star')
      ]
    },
    {
      id: 'makosh',
      title: 'Макошь',
      actions: [
        mailAction('analyze', 'Analyze', 'Refresh Макошь analysis for this message.', 'tabler:brain', 'accent', 'communication.mail.analyze'),
        mailAction('translate', 'Translate', 'Translate this message for review.', 'tabler:language', 'accent', 'communication.mail.translate'),
        mailAction('extract-tasks', 'Extract tasks', 'Extract task candidates from this message.', 'tabler:checkbox', 'accent', 'communication.mail.extract_tasks'),
        mailAction('extract-notes', 'Extract notes', 'Extract note candidates from this message.', 'tabler:notes', 'accent', 'communication.mail.extract_notes')
      ]
    },
    {
      id: 'create',
      title: 'Create',
      actions: [
        mailAction('create-task', 'Create task', 'Create task candidates from this message.', 'tabler:checkbox', 'success', 'communication.mail.create_task'),
        mailAction('create-document', 'Create note', 'Create note candidates from this message.', 'tabler:file-plus', 'success', 'communication.mail.create_note')
      ]
    },
    {
      id: 'evidence',
      title: 'Evidence',
      actions: [
        mailAction('auth-check', 'Auth check', 'Review SPF, DKIM and DMARC evidence.', 'tabler:shield-check', 'warning', 'communication.mail.auth_check'),
        mailAction('spf-dkim', 'SPF/DKIM', 'Review sender authentication evidence.', 'tabler:shield-lock', 'warning', 'communication.mail.signature_check'),
        mailAction('export-md', 'Export MD', 'Export this message as Markdown.', 'tabler:markdown', undefined, 'communication.mail.export.md'),
        mailAction('export-eml', 'Export EML', 'Export this message as EML.', 'tabler:file-type-eml', undefined, 'communication.mail.export.eml'),
        mailAction('export-json', 'Export JSON', 'Export this message as JSON.', 'tabler:braces', undefined, 'communication.mail.export.json')
      ]
    },
    {
      id: 'danger',
      title: 'Danger',
      actions: [
        mailAction('delete-provider', 'Delete provider', 'Request provider-side deletion for this message.', 'tabler:trash', 'danger', 'communication.mail.provider_delete')
      ]
    }
  ]
}

function spamStateAction(
  source: CommunicationMessageDetailItem | CommunicationMessageSummary
): CommunicationMessageActionModel {
  if (source.workflow_state === 'spam') {
    return mailAction(
      'mark-not-spam',
      'Not spam',
      'Return this message to the inbox and synchronize the provider.',
      'tabler:mail-check',
      'success',
      'communication.mail.workflow.not_spam'
    )
  }
  return mailAction(
    'mark-spam',
    'Mark spam',
    'Move this message into spam workflow state.',
    'tabler:mail-x',
    'warning',
    'communication.mail.workflow.spam'
  )
}

function mailAction(
  id: string,
  label: string,
  description: string,
  icon: string,
  tone?: CommunicationMessageActionModel['tone'],
  contract?: string
): CommunicationMessageActionModel {
  return { id, label, description, icon, tone, contract }
}

function importantLabel(source: CommunicationMessageDetailItem | CommunicationMessageSummary): string {
  return (source.importance_score ?? 0) >= 75 ? 'Unmark important' : 'Mark important'
}

function starLabel(source: CommunicationMessageDetailItem | CommunicationMessageSummary): string {
  return source.message_metadata?.starred === true ? 'Unstar message' : 'Star message'
}

function providerFlagActionDescription(
  synchronizedDescription: string,
  providerFlagMutationAvailable: boolean
): string {
  if (providerFlagMutationAvailable) return synchronizedDescription

  return 'Update Макошь local state. Provider sync is unavailable until this account has mail flag permissions.'
}
