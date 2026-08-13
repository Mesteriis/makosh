import { createDomainSurface } from '../../domainSurface'

const surfacePath = 'frontend/src/domains/calendar/queries/useCalendarPageSurface.ts'

export function useCalendarSurface() {
  return createDomainSurface({
    surfaceId: 'calendar',
    labelKey: 'Calendar',
    status: 'active',
    ownerLayer: 'domain',
    surfacePath,
    capabilities: [
      {
        id: 'calendar-agenda',
        labelKey: 'Agenda',
        descriptionKey: 'Owner-local Calendar lifecycle and bounded event queries.',
        icon: 'tabler:calendar-event',
        status: 'active',
        kind: 'query',
        contract: 'CalendarQueryService.List/Search/Get'
      },
      {
        id: 'calendar-create-event',
        labelKey: 'Create event',
        descriptionKey: 'Typed owner event creation and lifecycle mutation.',
        icon: 'tabler:calendar-plus',
        status: 'active',
        kind: 'command',
        contract: 'CalendarCommandService.Create/Update/SetState'
      },
      {
        id: 'calendar-reminders',
        labelKey: 'Reminders',
        descriptionKey: 'Durable Scheduler-backed reminders and immutable outcomes.',
        icon: 'tabler:bell',
        status: 'active',
        kind: 'command',
        contract: 'CalendarCommandService.AddReminder/RemoveReminder/RecordOutcome'
      }
    ],
    childSurfaces: [
      {
        id: 'calendar-agenda',
        labelKey: 'Agenda',
        status: 'active',
        surfacePath,
        capabilityIds: ['calendar-agenda']
      },
      {
        id: 'calendar-create',
        labelKey: 'Create',
        status: 'active',
        surfacePath,
        capabilityIds: ['calendar-create-event']
      },
      {
        id: 'calendar-reminders',
        labelKey: 'Reminders',
        status: 'active',
        surfacePath,
        capabilityIds: ['calendar-reminders']
      }
    ]
  })
}
