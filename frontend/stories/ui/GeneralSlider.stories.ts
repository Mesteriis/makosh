import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { RangeSlider, Slider } from '@/shared/ui'
import { storybookLocaleFromGlobals } from './storybook-i18n'
import { generalStoryCopy } from './general-story-copy'

const meta = {
	title: 'Макошь UI/General/Slider',
	component: Slider,
	render: (_args, context) => ({
		components: { RangeSlider, Slider },
		data() {
			return {
				copy: generalStoryCopy(storybookLocaleFromGlobals(context.globals)),
				value: 68,
				range: { min: 30, max: 88 }
			}
		},
		template: `
			<section class="storybook-canvas">
				<div class="storybook-section storybook-narrow">
					<h2>{{ copy.controls.slider }}</h2>
					<Slider v-model="value" :label="copy.form.threshold" :min="0" :max="100" />
					<RangeSlider v-model="range" :label="copy.form.confidence" :min="0" :max="100" />
				</div>
			</section>
		`
	})
} satisfies Meta<typeof Slider>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {}
