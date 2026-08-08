// Historical pre-clean-room provider surface. It is not part of the active client graph.
import { createCommunicationChannelSurface } from './communicationChannelSurface'

export function useMailCommunicationsSurface() {
  return createCommunicationChannelSurface({
    channelId: 'mail',
    labelKey: 'Mail',
    status: 'active',
    businessQueryRoot: ['communications', 'mail'] as const,
    runtimeQueryRoot: ['integrations', 'mail', 'runtime'] as const,
    surfacePath: 'frontend/src/domains/communications/queries/useCommunicationsPageSurface.ts',
    capabilityNotes: [
      'Provider-neutral message list and thread orchestration live in the Communications page surface.',
      'Mail sync settings and runtime status remain under Communications-owned surfaces.'
    ],
    capabilityGroups: [
      {
        id: 'mail-common',
        labelKey: 'Mail workspace',
        menuLabelKey: 'Open mail workspace capabilities',
        icon: 'tabler:mail',
        status: 'available',
        capabilities: [
          {
            id: 'mail-list',
            labelKey: 'Mail list and folders',
            descriptionKey: 'Provider-neutral list, account selection, saved searches and folder state.',
            icon: 'tabler:list-details',
            status: 'available',
            kind: 'query',
            contract: 'communications.mail.list'
          },
          {
            id: 'mail-actions',
            labelKey: 'Mail message actions',
            descriptionKey: 'Reply, forward, state, labels, evidence and provider delete actions.',
            icon: 'tabler:mail-forward',
            status: 'available',
            kind: 'command',
            contract: 'mailActionQueries'
          },
          {
            id: 'mail-inspector',
            labelKey: 'Mail Макошь inspector',
            descriptionKey: 'Candidates, extracted entities, evidence and suggested actions.',
            icon: 'tabler:sparkles',
            status: 'available',
            kind: 'inspector',
            contract: 'mailInspectorModel'
          }
        ]
      }
    ]
  })
}
