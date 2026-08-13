import { computed } from 'vue'
import { useCalendarStore } from '../stores/calendar'

export function useCalendarPageSurface() {
  const store = useCalendarStore()
  return {
    events: computed(() => store.events),
    selectedEvent: computed(() => store.selectedEvent),
    participants: computed(() => store.participants),
    reminders: computed(() => store.reminders),
    outcomes: computed(() => store.outcomes),
    searchQuery: computed(() => store.searchQuery),
    error: computed(() => store.error),
    isLoading: computed(() => store.isLoading),
    mutatingEventId: computed(() => store.mutatingEventId),
    scheduledEvents: computed(() => store.scheduledEvents),
    terminalEvents: computed(() => store.terminalEvents),
    loadEvents: store.loadAll,
    search: store.search,
    select: store.select,
    createEvent: store.createEvent,
    updateEvent: store.updateEvent,
    setEventState: store.setEventState,
    addParticipant: store.addParticipant,
    updateParticipant: store.updateParticipant,
    removeParticipant: store.removeParticipant,
    setConstraints: store.setConstraints,
    addReminder: store.addReminder,
    removeReminder: store.removeReminder,
    recordOutcome: store.recordOutcome,
    store
  }
}
