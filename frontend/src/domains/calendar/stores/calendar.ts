import { computed, ref } from 'vue'
import { defineStore } from 'pinia'
import {
  CalendarEventStateV1,
  CalendarOutcomeKindV1,
  CalendarParticipantResponseV1,
  CalendarParticipantRoleV1,
  type CalendarEventV1,
  type CalendarOutcomeV1,
  type CalendarParticipantV1,
  type CalendarReminderV1,
  type TimestampV1
} from '../../../gen/makosh/calendar/client/v1/calendar_pb'
import {
  getCalendarCommandClient,
  getCalendarQueryClient
} from '../../../platform/connect/calendarClient'

const PAGE_LIMIT = 50

export const useCalendarStore = defineStore('calendar-owner', () => {
  const events = ref<CalendarEventV1[]>([])
  const selectedEvent = ref<CalendarEventV1>()
  const participants = ref<CalendarParticipantV1[]>([])
  const reminders = ref<CalendarReminderV1[]>([])
  const outcomes = ref<CalendarOutcomeV1[]>([])
  const searchQuery = ref('')
  const error = ref('')
  const isLoading = ref(false)
  const mutatingEventId = ref<string | null>(null)

  const scheduledEvents = computed(() => events.value.filter((event) =>
    event.state === CalendarEventStateV1.CALENDAR_EVENT_STATE_SCHEDULED
  ))
  const terminalEvents = computed(() => events.value.filter((event) =>
    event.state === CalendarEventStateV1.CALENDAR_EVENT_STATE_COMPLETED
      || event.state === CalendarEventStateV1.CALENDAR_EVENT_STATE_CANCELLED
  ))

  async function loadAll(): Promise<void> {
    await loadPages('')
  }

  async function search(query: string): Promise<void> {
    const normalized = query.trim()
    searchQuery.value = normalized
    await loadPages(normalized)
  }

  async function select(event: CalendarEventV1): Promise<void> {
    isLoading.value = true
    error.value = ''
    try {
      selectedEvent.value = await getCalendarQueryClient().get({
        logicalOwnerId: '',
        calendarEventId: event.calendarEventId
      })
      await loadChildren(selectedEvent.value)
    } catch (cause) {
      error.value = message(cause)
      throw cause
    } finally {
      isLoading.value = false
    }
  }

  async function createEvent(input: {
    title: string
    description: string
    startsAt: Date
    endsAt: Date
    timezone: string
  }): Promise<void> {
    await run(null, async () => {
      const result = await getCalendarCommandClient().create({
        operationId: randomId16(),
        logicalOwnerId: '',
        title: input.title,
        description: input.description,
        startsAt: timestamp(input.startsAt),
        endsAt: timestamp(input.endsAt),
        timezone: input.timezone,
        createdAt: timestamp(new Date())
      })
      replaceResult(result.event)
    })
  }

  async function updateEvent(event: CalendarEventV1, input: {
    title?: string
    description?: string
    startsAt?: Date
    endsAt?: Date
    timezone?: string
  }): Promise<void> {
    await mutate(event, () => getCalendarCommandClient().update({
      operationId: randomId16(),
      calendarEventId: event.calendarEventId,
      logicalOwnerId: '',
      expectedEventRevision: event.eventRevision,
      title: input.title,
      description: input.description,
      startsAt: input.startsAt ? timestamp(input.startsAt) : undefined,
      endsAt: input.endsAt ? timestamp(input.endsAt) : undefined,
      timezone: input.timezone,
      updatedAt: timestamp(new Date())
    }))
  }

  async function setEventState(event: CalendarEventV1, state: CalendarEventStateV1): Promise<void> {
    await mutate(event, () => getCalendarCommandClient().setState({
      operationId: randomId16(),
      calendarEventId: event.calendarEventId,
      logicalOwnerId: '',
      expectedEventRevision: event.eventRevision,
      state,
      changedAt: timestamp(new Date())
    }))
  }

  async function addParticipant(event: CalendarEventV1, input: {
    displayName: string
    address: string
    role: CalendarParticipantRoleV1
  }): Promise<void> {
    await mutate(event, () => getCalendarCommandClient().addParticipant({
      operationId: randomId16(),
      calendarEventId: event.calendarEventId,
      logicalOwnerId: '',
      expectedEventRevision: event.eventRevision,
      displayName: input.displayName,
      address: input.address,
      role: input.role,
      response: CalendarParticipantResponseV1.CALENDAR_PARTICIPANT_RESPONSE_PENDING,
      changedAt: timestamp(new Date())
    }))
  }

  async function updateParticipant(
    event: CalendarEventV1,
    participant: CalendarParticipantV1,
    response: CalendarParticipantResponseV1
  ): Promise<void> {
    await mutate(event, () => getCalendarCommandClient().updateParticipant({
      operationId: randomId16(),
      calendarEventId: event.calendarEventId,
      logicalOwnerId: '',
      expectedEventRevision: event.eventRevision,
      participantId: participant.participantId,
      response,
      changedAt: timestamp(new Date())
    }))
  }

  async function removeParticipant(event: CalendarEventV1, participant: CalendarParticipantV1): Promise<void> {
    await mutate(event, () => getCalendarCommandClient().removeParticipant({
      operationId: randomId16(),
      calendarEventId: event.calendarEventId,
      logicalOwnerId: '',
      expectedEventRevision: event.eventRevision,
      participantId: participant.participantId,
      changedAt: timestamp(new Date())
    }))
  }

  async function setConstraints(event: CalendarEventV1, input: {
    earliestStart: Date
    latestEnd: Date
    minimumDurationMinutes: number
    timezone: string
  }): Promise<void> {
    await mutate(event, () => getCalendarCommandClient().setConstraints({
      operationId: randomId16(),
      calendarEventId: event.calendarEventId,
      logicalOwnerId: '',
      expectedEventRevision: event.eventRevision,
      earliestStart: timestamp(input.earliestStart),
      latestEnd: timestamp(input.latestEnd),
      minimumDurationMinutes: input.minimumDurationMinutes,
      timezone: input.timezone,
      changedAt: timestamp(new Date())
    }))
  }

  async function addReminder(event: CalendarEventV1, dueAt: Date): Promise<void> {
    await mutate(event, () => getCalendarCommandClient().addReminder({
      operationId: randomId16(),
      calendarEventId: event.calendarEventId,
      logicalOwnerId: '',
      expectedEventRevision: event.eventRevision,
      dueAt: timestamp(dueAt),
      changedAt: timestamp(new Date())
    }))
  }

  async function removeReminder(event: CalendarEventV1, reminder: CalendarReminderV1): Promise<void> {
    await mutate(event, () => getCalendarCommandClient().removeReminder({
      operationId: randomId16(),
      calendarEventId: event.calendarEventId,
      logicalOwnerId: '',
      expectedEventRevision: event.eventRevision,
      reminderId: reminder.reminderId,
      changedAt: timestamp(new Date())
    }))
  }

  async function recordOutcome(
    event: CalendarEventV1,
    kind: CalendarOutcomeKindV1,
    note: string
  ): Promise<void> {
    await mutate(event, () => getCalendarCommandClient().recordOutcome({
      operationId: randomId16(),
      calendarEventId: event.calendarEventId,
      logicalOwnerId: '',
      expectedEventRevision: event.eventRevision,
      kind,
      note,
      recordedAt: timestamp(new Date())
    }))
  }

  async function loadPages(query: string): Promise<void> {
    isLoading.value = true
    error.value = ''
    try {
      const loaded: CalendarEventV1[] = []
      let cursor: Uint8Array<ArrayBufferLike> = new Uint8Array()
      do {
        const page = query
          ? await getCalendarQueryClient().search({
              logicalOwnerId: '', query, afterCalendarEventId: cursor, limit: PAGE_LIMIT
            })
          : await getCalendarQueryClient().list({
              logicalOwnerId: '', afterCalendarEventId: cursor, limit: PAGE_LIMIT
            })
        loaded.push(...page.events)
        cursor = page.nextAfterCalendarEventId
      } while (cursor.length > 0)
      events.value = loaded
    } catch (cause) {
      error.value = message(cause)
    } finally {
      isLoading.value = false
    }
  }

  async function loadChildren(event: CalendarEventV1): Promise<void> {
    const eventId = event.calendarEventId
    const loadedParticipants: CalendarParticipantV1[] = []
    const loadedReminders: CalendarReminderV1[] = []
    const loadedOutcomes: CalendarOutcomeV1[] = []
    let participantCursor: Uint8Array<ArrayBufferLike> = new Uint8Array()
    let reminderCursor: Uint8Array<ArrayBufferLike> = new Uint8Array()
    let outcomeCursor: Uint8Array<ArrayBufferLike> = new Uint8Array()
    do {
      const page = await getCalendarQueryClient().listParticipants({
        logicalOwnerId: '', calendarEventId: eventId, afterId: participantCursor, limit: PAGE_LIMIT
      })
      loadedParticipants.push(...page.participants)
      participantCursor = page.nextAfterParticipantId
    } while (participantCursor.length > 0)
    do {
      const page = await getCalendarQueryClient().listReminders({
        logicalOwnerId: '', calendarEventId: eventId, afterId: reminderCursor, limit: PAGE_LIMIT
      })
      loadedReminders.push(...page.reminders)
      reminderCursor = page.nextAfterReminderId
    } while (reminderCursor.length > 0)
    do {
      const page = await getCalendarQueryClient().listOutcomes({
        logicalOwnerId: '', calendarEventId: eventId, afterId: outcomeCursor, limit: PAGE_LIMIT
      })
      loadedOutcomes.push(...page.outcomes)
      outcomeCursor = page.nextAfterOutcomeId
    } while (outcomeCursor.length > 0)
    participants.value = loadedParticipants
    reminders.value = loadedReminders
    outcomes.value = loadedOutcomes
  }

  async function mutate(
    event: CalendarEventV1,
    operation: () => Promise<{ event?: CalendarEventV1 }>
  ): Promise<void> {
    await run(hex(event.calendarEventId), async () => {
      const result = await operation()
      replaceResult(result.event)
      if (result.event) await loadChildren(result.event)
    })
  }

  async function run(eventId: string | null, operation: () => Promise<void>): Promise<void> {
    mutatingEventId.value = eventId
    error.value = ''
    try {
      await operation()
    } catch (cause) {
      error.value = message(cause)
      throw cause
    } finally {
      mutatingEventId.value = null
    }
  }

  function replaceResult(event: CalendarEventV1 | undefined): void {
    if (!event) throw new Error('calendar_invalid_response')
    const index = events.value.findIndex((value) => sameBytes(value.calendarEventId, event.calendarEventId))
    if (index === -1) events.value.push(event)
    else events.value[index] = event
    events.value.sort((left, right) => compareBytes(left.calendarEventId, right.calendarEventId))
    if (selectedEvent.value && sameBytes(selectedEvent.value.calendarEventId, event.calendarEventId)) {
      selectedEvent.value = event
    }
  }

  return {
    events,
    selectedEvent,
    participants,
    reminders,
    outcomes,
    searchQuery,
    error,
    isLoading,
    mutatingEventId,
    scheduledEvents,
    terminalEvents,
    loadAll,
    search,
    select,
    createEvent,
    updateEvent,
    setEventState,
    addParticipant,
    updateParticipant,
    removeParticipant,
    setConstraints,
    addReminder,
    removeReminder,
    recordOutcome
  }
})

export function timestamp(value: Date): TimestampV1 {
  const milliseconds = value.getTime()
  return {
    $typeName: 'makosh.calendar.client.v1.TimestampV1',
    unixSeconds: BigInt(Math.floor(milliseconds / 1_000)),
    nanos: Math.trunc(milliseconds % 1_000) * 1_000_000
  }
}

export function dateFromTimestamp(value: TimestampV1 | undefined): Date | undefined {
  if (!value) return undefined
  return new Date(Number(value.unixSeconds) * 1_000 + Math.trunc(value.nanos / 1_000_000))
}

export function hex(value: Uint8Array): string {
  return Array.from(value, (byte) => byte.toString(16).padStart(2, '0')).join('')
}

function randomId16(): Uint8Array {
  return globalThis.crypto.getRandomValues(new Uint8Array(16))
}

function sameBytes(left: Uint8Array, right: Uint8Array): boolean {
  return left.length === right.length && left.every((value, index) => value === right[index])
}

function compareBytes(left: Uint8Array, right: Uint8Array): number {
  const length = Math.min(left.length, right.length)
  for (let index = 0; index < length; index += 1) {
    const comparison = (left[index] ?? 0) - (right[index] ?? 0)
    if (comparison !== 0) return comparison
  }
  return left.length - right.length
}

function message(cause: unknown): string {
  return cause instanceof Error ? cause.message : 'calendar_unavailable'
}
