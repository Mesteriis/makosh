import { describe, expect, it } from 'vitest'
import { existsSync, readFileSync } from 'node:fs'

describe('Calendar owner product boundary', () => {
  it('renders the typed Calendar workspace without the retired REST layer', () => {
    const view = readFileSync(new URL('./CalendarWorkspaceView.vue', import.meta.url), 'utf8')
    const store = readFileSync(new URL('../stores/calendar.ts', import.meta.url), 'utf8')

    expect(existsSync(new URL('../api', import.meta.url))).toBe(false)
    expect(view).toContain('useCalendarPageSurface')
    expect(view).toContain('Calendar')
    expect(store).toContain('getCalendarQueryClient')
    expect(store).toContain('getCalendarCommandClient')
    expect(store).toContain('addReminder')
    expect(store).toContain('recordOutcome')
    expect(store).not.toContain('ApiClient')
  })
})
