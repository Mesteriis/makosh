import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { FormField, FormLabel, TokenInput } from '@/shared/ui'
import { storybookLocaleFromGlobals } from './storybook-i18n'
import { generalStoryCopy } from './general-story-copy'

const meta = {
	title: 'Макошь UI/General/Token Input',
	component: TokenInput,
	render: (_args, context) => ({
		components: { FormField, FormLabel, TokenInput },
		data() {
			const copy = generalStoryCopy(storybookLocaleFromGlobals(context.globals))
			return { copy, value: copy.tokens }
		},
		template: `
			<section class="storybook-canvas">
				<div class="storybook-section storybook-narrow">
					<h2>{{ copy.controls.tokenInput }}</h2>
					<FormField>
						<FormLabel>{{ copy.controls.tokenInput }}</FormLabel>
						<TokenInput v-model="value" :aria-label="copy.controls.tokenInput" :placeholder="copy.form.tokensPlaceholder" :remove-label="copy.form.remove" />
					</FormField>
				</div>
			</section>
		`
	})
} satisfies Meta<typeof TokenInput>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {}
