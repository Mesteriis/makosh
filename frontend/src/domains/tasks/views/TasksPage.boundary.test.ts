import { describe, expect, it } from 'vitest'
import { existsSync, readFileSync } from 'node:fs'

describe('Tasks product boundary', () => {
  it('renders only typed Tasks owner truth and has no legacy REST adapter', () => {
    const store = readFileSync(new URL('../stores/tasks.ts', import.meta.url), 'utf8')
    const view = readFileSync(new URL('./TasksWorkspaceView.vue', import.meta.url), 'utf8')

    expect(existsSync(new URL('../api/tasks.ts', import.meta.url))).toBe(false)
    expect(store).toContain('getTasksCommandClient')
    expect(store).toContain('getTasksQueryClient')
    expect(store).not.toContain('/api/v1/tasks')
    expect(store).not.toContain('Obligation')
    expect(store).not.toContain('Decision')
    expect(view).toContain('surface.setTaskState')
    expect(view).toContain('surface.setTaskPriority')
    expect(view).toContain('surface.updateChecklistItem')
  })
})
