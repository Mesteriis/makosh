// Historical pre-clean-room provider surface. It is not part of the active client graph.
import { createCommunicationSubSurface } from './communicationChannelSurface'

export function useMeetingsCommunicationsSurface() {
  return createCommunicationSubSurface({
    channelId: 'meetings',
    labelKey: 'Meetings',
    status: 'facade',
    businessQueryRoot: ['communications', 'meetings'] as const,
    surfacePath: 'frontend/src/domains/communications/queries/useMeetingsCommunicationsSurface.ts',
    capabilityNotes: [
      'Meetings are the scheduled communication surface: calendar context plus provider meeting rooms.',
      'Meeting creation and permanent room links stay provider-neutral until routed to Zoom, Telemost or future providers.'
    ],
    capabilityGroups: [
      {
        id: 'meetings-workspace',
        labelKey: 'Meetings workspace',
        menuLabelKey: 'Open meeting workspace capabilities',
        icon: 'tabler:calendar-time',
        status: 'facade',
        capabilities: [
          {
            id: 'meetings-permanent-rooms',
            labelKey: 'Permanent rooms',
            descriptionKey: 'Pinned Zoom, Telemost or provider room links for recurring owner workflows.',
            icon: 'tabler:pin',
            status: 'partial',
            kind: 'query',
            contract: 'communications.meetings.permanent_rooms'
          },
          {
            id: 'meetings-schedule',
            labelKey: 'Schedule meeting',
            descriptionKey: 'Meeting creation intent with provider selection and calendar context.',
            icon: 'tabler:calendar-plus',
            status: 'facade',
            kind: 'command',
            contract: 'communications.meetings.schedule'
          },
          {
            id: 'meetings-context',
            labelKey: 'Meeting context',
            descriptionKey: 'Agenda, participants, source evidence and pre-meeting Макошь context.',
            icon: 'tabler:notes',
            status: 'facade',
            kind: 'projection',
            contract: 'communications.meetings.context'
          }
        ]
      }
    ]
  })
}
