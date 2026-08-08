import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('Tabs compatibility API', () => {
  it('renders trigger buttons from tabs/active props and emits select events', () => {
    const source = readFileSync(new URL('./Tabs.vue', import.meta.url), 'utf8')

    expect(source).toContain('tabs?: МакошьTab[]')
    expect(source).toContain('active?: string')
    expect(source).toContain('select: [value: string]')
    expect(source).toContain('<TabsTrigger')
    expect(source).toContain('v-for="tab in tabs"')
    expect(source).toContain('@update:model-value="handleUpdateModelValue"')
  })
})
