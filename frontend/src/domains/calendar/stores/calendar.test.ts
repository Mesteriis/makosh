import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import {
  CalendarEventStateV1,
  type CalendarEventV1
} from '../../../gen/makosh/calendar/client/v1/calendar_pb'

const clients = vi.hoisted(() => ({
  query: {
    list: vi.fn(), search: vi.fn(), get: vi.fn(), listParticipants: vi.fn(),
    listReminders: vi.fn(), listOutcomes: vi.fn()
  },
  command: {
    create: vi.fn(), update: vi.fn(), setState: vi.fn(), addParticipant: vi.fn(),
    updateParticipant: vi.fn(), removeParticipant: vi.fn(), setConstraints: vi.fn(),
    addReminder: vi.fn(), removeReminder: vi.fn(), recordOutcome: vi.fn()
  }
}))

vi.mock('../../../platform/connect/calendarClient', () => ({
  getCalendarQueryClient: () => clients.query,
  getCalendarCommandClient: () => clients.command
}))

import { useCalendarStore } from './calendar'

describe('typed Calendar owner store', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
    clients.query.listParticipants.mockResolvedValue({ participants: [], nextAfterParticipantId: new Uint8Array() })
    clients.query.listReminders.mockResolvedValue({ reminders: [], nextAfterReminderId: new Uint8Array() })
    clients.query.listOutcomes.mockResolvedValue({ outcomes: [], nextAfterOutcomeId: new Uint8Array() })
  })

  it('loads all bounded pages using the exclusive last-returned cursor', async () => {
    clients.query.list
      .mockResolvedValueOnce({ events: [event(1)], nextAfterCalendarEventId: id(1) })
      .mockResolvedValueOnce({ events: [event(2)], nextAfterCalendarEventId: new Uint8Array() })
    const store = useCalendarStore()

    await store.loadAll()

    expect(store.events.map((value) => value.calendarEventId)).toEqual([id(1), id(2)])
    expect(clients.query.list.mock.calls).toEqual([
      [{ logicalOwnerId: '', afterCalendarEventId: new Uint8Array(), limit: 50 }],
      [{ logicalOwnerId: '', afterCalendarEventId: id(1), limit: 50 }]
    ])
  })

  it('dispatches lifecycle and reminder commands with exact current revisions', async () => {
    const initial = event(1)
    const revised = { ...initial, eventRevision: 2n }
    clients.query.list.mockResolvedValue({ events: [initial], nextAfterCalendarEventId: new Uint8Array() })
    clients.command.setState.mockResolvedValue({ event: revised })
    clients.command.addReminder.mockResolvedValue({ event: { ...revised, eventRevision: 3n } })
    const store = useCalendarStore()
    await store.loadAll()

    await store.setEventState(store.events[0]!, CalendarEventStateV1.CALENDAR_EVENT_STATE_COMPLETED)
    await store.addReminder(store.events[0]!, new Date('2030-01-02T03:04:05Z'))

    expect(clients.command.setState.mock.calls[0]?.[0]).toMatchObject({
      calendarEventId: id(1), expectedEventRevision: 1n,
      state: CalendarEventStateV1.CALENDAR_EVENT_STATE_COMPLETED
    })
    expect(clients.command.addReminder.mock.calls[0]?.[0]).toMatchObject({
      calendarEventId: id(1), expectedEventRevision: 2n
    })
    expect(store.events[0]?.eventRevision).toBe(3n)
  })
})

function event(seed: number): CalendarEventV1 {
  return {
    $typeName: 'makosh.calendar.client.v1.CalendarEventV1',
    calendarEventId: id(seed),
    logicalOwnerId: 'owner-1',
    title: `Event ${seed}`,
    description: '',
    timezone: 'UTC',
    state: CalendarEventStateV1.CALENDAR_EVENT_STATE_SCHEDULED,
    eventRevision: 1n
  }
}

function id(seed: number): Uint8Array {
  return new Uint8Array(16).fill(seed)
}
