import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { ToggleGroup } from '@/shared/ui'
import { storybookLocaleFromGlobals } from './storybook-i18n'
import { generalStoryCopy } from './general-story-copy'

const meta = {
	title: 'Макошь UI/General/Toggle Group',
	component: ToggleGroup,
	render: (_args, context) => ({
		components: { ToggleGroup },
		data() {
			const copy = generalStoryCopy(storybookLocaleFromGlobals(context.globals))
			return {
				copy,
				languages: [
					{ value: 'ru', label: 'Русский' },
					{ value: 'en', label: 'English' }
				],
				selected: 'review',
				selectedLanguage: 'ru',
				selectedMany: ['review', 'evidence']
			}
		},
		template: `
			<section class="storybook-canvas">
				<div class="storybook-section">
					<h2>{{ copy.controls.toggleGroup }}</h2>
					<ToggleGroup v-model="selected" :aria-label="copy.controls.toggleGroup" :items="copy.toggles" />
					<ToggleGroup v-model="selectedMany" multiple :aria-label="copy.controls.toggleGroup" :items="copy.toggles" />
					<ToggleGroup
						v-model="selectedLanguage"
						class="makosh-toggle-group--tabs"
						aria-label="Language"
						:items="languages"
					/>
				</div>
			</section>
		`
	})
} satisfies Meta<typeof ToggleGroup>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {}
