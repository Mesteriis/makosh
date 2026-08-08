import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('Steps modal wizard contract', () => {
	it('keeps step content generic through named slots', () => {
		const source = readFileSync(new URL('./Steps.vue', import.meta.url), 'utf8')
		const overlayStyles = readFileSync(new URL('./styles/overlays.css', import.meta.url), 'utf8')
		const storySource = readFileSync(new URL('../../../stories/ui/GeneralSteps.stories.ts', import.meta.url), 'utf8')

		expect(source).toContain('const activeSlotName = computed(() => `step-${activeStep.value}`)')
		expect(source).toContain('<slot :name="activeSlotName" v-bind="slotProps">')
		expect(source).toContain('<Transition :name="transitionName" mode="out-in">')
		expect(source).toContain("transitionName.value = value > previousValue ? 'makosh-steps-slide-forward' : 'makosh-steps-slide-backward'")
		expect(source).toContain('class="makosh-steps__dots"')
		expect(source).toContain('class="makosh-steps__dock"')
		expect(source).toContain('class="makosh-steps__dock-next"')
		expect(source).toContain(':show-close="false"')
		expect(source).toContain('showCancel: false')
		expect(source).not.toContain('class="makosh-steps__rail"')
		expect(source).toContain("'update:step': [value: number]")
		expect(overlayStyles).toContain('justify-self: center;')
		expect(storySource).toContain(':step-count="3"')
		expect(storySource).toContain('<template #step-1>')
		expect(storySource).toContain('<template #step-2>')
		expect(storySource).toContain('<template #step-3>')
	})
})
