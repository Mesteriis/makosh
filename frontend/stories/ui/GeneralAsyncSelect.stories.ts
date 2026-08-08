import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { AsyncSelect, FormField, FormLabel } from '@/shared/ui'
import { storybookLocaleFromGlobals, storybookText } from './storybook-i18n'

const meta = {
	title: 'Макошь UI/General/Async Select',
	render: (_args, context) => ({
		components: { AsyncSelect, FormField, FormLabel },
		data() {
			const text = storybookText(storybookLocaleFromGlobals(context.globals))
			return {
				text,
				loadedValue: 'communications',
				loadingValue: '',
				errorValue: ''
			}
		},
		template: `
			<section class="storybook-canvas">
				<div class="storybook-section storybook-narrow">
					<h2>{{ text.selection.asyncSelect }}</h2>
					<FormField>
						<FormLabel>{{ text.selection.asyncSelect }}</FormLabel>
						<AsyncSelect
							v-model="loadedValue"
							:empty-label="text.selection.empty"
							:options="text.selection.options"
							:placeholder="text.selection.placeholder"
							:retry-label="text.selection.retry"
							:search-placeholder="text.selection.searchPlaceholder"
						/>
					</FormField>
					<FormField>
						<FormLabel>{{ text.selection.loading }}</FormLabel>
						<AsyncSelect
							v-model="loadingValue"
							loading
							:loading-label="text.selection.loading"
							:options="[]"
							:placeholder="text.selection.placeholder"
							:retry-label="text.selection.retry"
						/>
					</FormField>
					<FormField>
						<FormLabel>{{ text.selection.error }}</FormLabel>
						<AsyncSelect
							v-model="errorValue"
							:error="text.selection.error"
							:options="[]"
							:placeholder="text.selection.placeholder"
							:retry-label="text.selection.retry"
						/>
					</FormField>
				</div>
			</section>
		`
	})
} satisfies Meta

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
	parameters: {
		a11y: {
			test: 'error'
		}
	}
}
