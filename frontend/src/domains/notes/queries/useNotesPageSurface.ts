import { computed } from 'vue'
import { useNotesQuery } from './useNotesQuery'
import type { NoteItem } from '../types/notes'

const fallbackNotes: NoteItem[] = [
  { title: 'Welcome to Notes', body: 'This is your personal notes workspace. Notes from connected sources will appear here.', source: 'Макошь', tag: '#reference', time: new Date().toISOString(), icon: 'tabler:notes' },
  { title: 'Meeting Notes Template', body: 'Use this template to capture key decisions, action items, and follow-ups from meetings.', source: 'Макошь', tag: '#meeting', time: new Date().toISOString(), icon: 'tabler:clipboard-list' },
  { title: 'Project Ideas', body: 'A collection of project ideas and brainstorming notes for future development.', source: 'Макошь', tag: '#idea', time: new Date().toISOString(), icon: 'tabler:lightbulb' },
  { title: 'Research Notes', body: 'Research findings and references organized by topic for easy retrieval.', source: 'Макошь', tag: '#research', time: new Date().toISOString(), icon: 'tabler:books' }
]

export function useNotesPageSurface() {
  const notesQuery = useNotesQuery()

  const notes = computed<NoteItem[]>(() => notesQuery.data.value?.items ?? fallbackNotes)

  return {
    fallbackNotes,
    notes,
    notesQuery
  }
}
