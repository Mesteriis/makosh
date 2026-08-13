<script setup lang="ts">
import { onMounted, reactive, ref } from 'vue'
import {
  CalendarEventStateV1,
  CalendarOutcomeKindV1,
  CalendarParticipantRoleV1,
  type CalendarEventV1
} from '../../../gen/makosh/calendar/client/v1/calendar_pb'
import { Button, Card, Input, Select } from '../../../shared/ui'
import { useCalendarPageSurface } from '../queries/useCalendarPageSurface'
import { dateFromTimestamp, hex } from '../stores/calendar'

const surface = useCalendarPageSurface()
const query = ref('')
const title = ref('')
const description = ref('')
const startsAt = ref('')
const endsAt = ref('')
const participantDraft = reactive({ displayName: '', address: '' })
const reminderDueAt = ref('')
const outcomeNote = ref('')

const stateOptions = [
  { value: String(CalendarEventStateV1.CALENDAR_EVENT_STATE_SCHEDULED), label: 'Scheduled' },
  { value: String(CalendarEventStateV1.CALENDAR_EVENT_STATE_COMPLETED), label: 'Completed' },
  { value: String(CalendarEventStateV1.CALENDAR_EVENT_STATE_CANCELLED), label: 'Cancelled' }
]

onMounted(() => { void surface.loadEvents() })

async function createEvent(): Promise<void> {
  if (!title.value.trim() || !startsAt.value || !endsAt.value) return
  await surface.createEvent({
    title: title.value.trim(),
    description: description.value.trim(),
    startsAt: new Date(startsAt.value),
    endsAt: new Date(endsAt.value),
    timezone: Intl.DateTimeFormat().resolvedOptions().timeZone || 'UTC'
  })
  title.value = ''
  description.value = ''
  startsAt.value = ''
  endsAt.value = ''
}

async function addParticipant(event: CalendarEventV1): Promise<void> {
  if (!participantDraft.displayName.trim() || !participantDraft.address.trim()) return
  await surface.addParticipant(event, {
    displayName: participantDraft.displayName.trim(),
    address: participantDraft.address.trim(),
    role: CalendarParticipantRoleV1.CALENDAR_PARTICIPANT_ROLE_REQUIRED
  })
  participantDraft.displayName = ''
  participantDraft.address = ''
}

async function addReminder(event: CalendarEventV1): Promise<void> {
  if (!reminderDueAt.value) return
  await surface.addReminder(event, new Date(reminderDueAt.value))
  reminderDueAt.value = ''
}

async function complete(event: CalendarEventV1): Promise<void> {
  await surface.recordOutcome(
    event,
    CalendarOutcomeKindV1.CALENDAR_OUTCOME_KIND_COMPLETED,
    outcomeNote.value.trim()
  )
  outcomeNote.value = ''
}

function isMutating(event: CalendarEventV1): boolean {
  return surface.mutatingEventId.value === hex(event.calendarEventId)
}

function displayTime(event: CalendarEventV1): string {
  const value = dateFromTimestamp(event.startsAt)
  return value ? value.toLocaleString() : 'Time unavailable'
}
</script>

<template>
  <main class="calendar-workspace" aria-label="Calendar">
    <header class="calendar-workspace__header">
      <div>
        <p class="calendar-workspace__eyebrow">OWNER CALENDAR</p>
        <h1>Calendar</h1>
        <p>{{ surface.scheduledEvents.value.length }} scheduled · {{ surface.terminalEvents.value.length }} closed</p>
      </div>
      <Button variant="secondary" icon="tabler:refresh" :loading="surface.isLoading.value" @click="surface.loadEvents">Refresh</Button>
    </header>

    <form class="calendar-workspace__search" @submit.prevent="surface.search(query)">
      <Input v-model="query" aria-label="Search Calendar" placeholder="Search owner events…" />
      <Button type="submit" variant="outline" icon="tabler:search">Search</Button>
      <Button v-if="surface.searchQuery.value" type="button" variant="ghost" @click="query = ''; surface.loadEvents()">Clear</Button>
    </form>

    <form class="calendar-workspace__create" @submit.prevent="createEvent">
      <Input v-model="title" aria-label="Calendar event title" placeholder="Event title" />
      <Input v-model="description" aria-label="Calendar event description" placeholder="Description" />
      <label>Starts <input v-model="startsAt" type="datetime-local" required /></label>
      <label>Ends <input v-model="endsAt" type="datetime-local" required /></label>
      <Button type="submit" icon="tabler:calendar-plus" :disabled="!title.trim() || !startsAt || !endsAt">Create event</Button>
    </form>

    <p v-if="surface.error.value" class="calendar-workspace__error" role="alert">{{ surface.error.value }}</p>
    <p v-if="surface.isLoading.value && surface.events.value.length === 0" aria-live="polite">Loading Calendar…</p>
    <p v-else-if="surface.events.value.length === 0" class="calendar-workspace__empty">No matching events.</p>

    <section v-else class="calendar-workspace__layout">
      <div class="calendar-workspace__list" aria-label="Calendar events">
        <Card
          v-for="event in surface.events.value"
          :key="hex(event.calendarEventId)"
          class="calendar-workspace__event"
          :selected="surface.selectedEvent.value !== undefined && hex(surface.selectedEvent.value.calendarEventId) === hex(event.calendarEventId)"
          @click="surface.select(event)"
        >
          <h2>{{ event.title }}</h2>
          <p>{{ displayTime(event) }} · {{ event.timezone }}</p>
          <small>Revision {{ event.eventRevision }}</small>
          <Select
            :model-value="String(event.state)"
            :options="stateOptions"
            aria-label="Calendar event state"
            :disabled="isMutating(event)"
            @click.stop
            @update:model-value="surface.setEventState(event, Number($event) as CalendarEventStateV1)"
          />
        </Card>
      </div>

      <Card v-if="surface.selectedEvent.value" class="calendar-workspace__detail">
        <h2>{{ surface.selectedEvent.value.title }}</h2>
        <p>{{ surface.selectedEvent.value.description || 'No description.' }}</p>

        <section>
          <h3>Participants</h3>
          <ul><li v-for="participant in surface.participants.value" :key="hex(participant.participantId)">{{ participant.displayName }} · {{ participant.address }}</li></ul>
          <form @submit.prevent="addParticipant(surface.selectedEvent.value)">
            <Input v-model="participantDraft.displayName" aria-label="Participant name" placeholder="Name" />
            <Input v-model="participantDraft.address" aria-label="Participant address" placeholder="Public address" />
            <Button type="submit" variant="outline" size="sm">Add participant</Button>
          </form>
        </section>

        <section>
          <h3>Reminders</h3>
          <ul><li v-for="reminder in surface.reminders.value" :key="hex(reminder.reminderId)">{{ dateFromTimestamp(reminder.dueAt)?.toLocaleString() }} · state {{ reminder.state }}</li></ul>
          <form @submit.prevent="addReminder(surface.selectedEvent.value)">
            <input v-model="reminderDueAt" aria-label="Reminder due time" type="datetime-local" required />
            <Button type="submit" variant="outline" size="sm">Schedule reminder</Button>
          </form>
        </section>

        <section>
          <h3>Outcomes</h3>
          <ul><li v-for="outcome in surface.outcomes.value" :key="hex(outcome.outcomeId)">{{ outcome.note || 'Recorded outcome' }}</li></ul>
          <form @submit.prevent="complete(surface.selectedEvent.value)">
            <Input v-model="outcomeNote" aria-label="Outcome note" placeholder="Outcome note" />
            <Button type="submit" variant="outline" size="sm">Record completed outcome</Button>
          </form>
        </section>
      </Card>
    </section>
  </main>
</template>

<style scoped>
.calendar-workspace { display: grid; gap: 1.25rem; width: min(78rem, 100%); margin: 0 auto; padding: 1.5rem; }
.calendar-workspace__header, .calendar-workspace__search, .calendar-workspace__create, .calendar-workspace__detail form { display: flex; gap: .75rem; align-items: center; }
.calendar-workspace__header { justify-content: space-between; align-items: flex-start; }
.calendar-workspace__header h1, .calendar-workspace__event h2, .calendar-workspace__detail h2, .calendar-workspace__detail h3 { margin: 0; }
.calendar-workspace__eyebrow { margin: 0 0 .25rem; font-size: .75rem; font-weight: 700; letter-spacing: .12em; }
.calendar-workspace__header p, .calendar-workspace__event p, .calendar-workspace__event small { color: var(--text-secondary); }
.calendar-workspace__search :deep(.makosh-input-wrapper), .calendar-workspace__create :deep(.makosh-input-wrapper), .calendar-workspace__detail :deep(.makosh-input-wrapper) { flex: 1; }
.calendar-workspace__create { flex-wrap: wrap; padding: 1rem; border: 1px solid var(--border-subtle); border-radius: .75rem; }
.calendar-workspace__create label { display: grid; gap: .25rem; font-size: .75rem; color: var(--text-secondary); }
.calendar-workspace input[type='datetime-local'] { padding: .625rem; color: inherit; background: var(--surface-raised); border: 1px solid var(--border-subtle); border-radius: .5rem; }
.calendar-workspace__layout { display: grid; grid-template-columns: minmax(18rem, 2fr) minmax(22rem, 3fr); gap: 1rem; align-items: start; }
.calendar-workspace__list, .calendar-workspace__detail { display: grid; gap: .75rem; }
.calendar-workspace__event { display: grid; gap: .5rem; padding: 1rem; cursor: pointer; }
.calendar-workspace__event--selected { outline: 2px solid var(--focus-ring); }
.calendar-workspace__detail { padding: 1rem; }
.calendar-workspace__detail section { display: grid; gap: .5rem; padding-top: .75rem; border-top: 1px solid var(--border-subtle); }
.calendar-workspace__detail ul { display: grid; gap: .375rem; margin: 0; padding-left: 1.25rem; }
.calendar-workspace__error { padding: .75rem 1rem; color: var(--status-error-text); background: var(--status-error-bg); border-radius: .75rem; }
.calendar-workspace__empty { padding: 3rem; text-align: center; color: var(--text-secondary); }
@media (max-width: 820px) { .calendar-workspace__layout { grid-template-columns: 1fr; } .calendar-workspace__header, .calendar-workspace__search, .calendar-workspace__create, .calendar-workspace__detail form { align-items: stretch; flex-direction: column; } }
</style>
