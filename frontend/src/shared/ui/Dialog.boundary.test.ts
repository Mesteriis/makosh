import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('Dialog controlled mode compatibility', () => {
  it('renders DialogTrigger only when a trigger slot is provided', () => {
    const source = readFileSync(new URL('./Dialog.vue', import.meta.url), 'utf8')

    expect(source).toContain('<DialogTrigger v-if="$slots.trigger" as-child>')
    expect(source).toContain('<DialogRoot :open="open" @update:open="(val) => emit(\'update:open\', val)">')
    expect(source).toContain('showClose: true')
    expect(source).toContain('closeOnInteractOutside: true')
    expect(source).toContain('function handleOutsideCloseEvent(event: Event): void')
    expect(source).toContain('event.preventDefault()')
    expect(source).toContain('@interact-outside="handleOutsideCloseEvent"')
    expect(source).toContain('@pointer-down-outside="handleOutsideCloseEvent"')
    expect(source).toContain('<slot name="chrome" />')
    expect(source).toContain('<DialogClose v-if="showClose" class="makosh-dialog-close" as-child>')
  })
})
