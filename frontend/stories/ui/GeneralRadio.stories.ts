import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { Radio, RadioGroup } from '@/shared/ui'
import { storybookLocaleFromGlobals } from './storybook-i18n'
import { generalStoryCopy } from './general-story-copy'

const meta = {
	title: 'Макошь UI/General/Radio',
	component: RadioGroup,
	render: (_args, context) => ({
		components: { Radio, RadioGroup },
		data() {
			return { copy: generalStoryCopy(storybookLocaleFromGlobals(context.globals)), value: 'evidence' }
		},
		template: `
			<section class="storybook-canvas">
				<div class="storybook-section">
					<h2>{{ copy.controls.radio }}</h2>
					<RadioGroup v-model="value" name="general-radio" :label="copy.form.confidence">
						<Radio value="review">{{ copy.toggles[0].label }}</Radio>
						<Radio value="evidence">{{ copy.toggles[1].label }}</Radio>
						<Radio value="memory">{{ copy.toggles[2].label }}</Radio>
					</RadioGroup>
				</div>
			</section>
		`
	})
} satisfies Meta<typeof RadioGroup>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {}
