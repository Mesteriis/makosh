import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { expect, within } from 'storybook/test'
import { HtmlPreview, KeyboardHint, RichTextEditor } from '@/shared/ui'
import { storybookLocaleFromGlobals, storybookText } from './storybook-i18n'

const meta = {
	title: 'Макошь UI/General/Editor',
	component: RichTextEditor
} satisfies Meta<typeof RichTextEditor>

export default meta
type Story = StoryObj<typeof meta>

export const ContextEditor: Story = {
	render: (_args, context) => ({
		components: {
			HtmlPreview,
			KeyboardHint,
			RichTextEditor
		},
		data() {
			const text = storybookText(storybookLocaleFromGlobals(context.globals))
			return {
				text,
				content: text.editor.initialHtml
			}
		},
		template: `
			<section class="storybook-canvas">
				<div class="storybook-section">
					<h2>{{ text.editor.title }}</h2>
					<p>{{ text.editor.description }}</p>
				</div>

				<div class="storybook-grid">
					<div class="storybook-section">
						<RichTextEditor
							v-model="content"
							:actions="text.editor.actions"
							:helper="text.editor.helper"
							:label="text.editor.label"
							:placeholder="text.editor.placeholder"
							:output-label="text.editor.outputLabel"
							:toolbar-label="text.editor.toolbarLabel"
							link-href="https://example.local/evidence/message-42"
							:max-length="720"
						/>
						<KeyboardHint :label="text.editor.keyboardLabel" :keys="['Meta', 'Enter']" />
					</div>

					<HtmlPreview
						:title="text.editor.previewTitle"
						format="html"
						sanitized
						:content="content"
						:empty-label="text.editor.previewEmpty"
					/>
				</div>
			</section>
		`
	}),
	play: async ({ canvasElement, globals }) => {
		const text = storybookText(storybookLocaleFromGlobals(globals))
		const codeBlockAction = text.editor.actions.find((action) => action.id === 'codeBlock')
		const canvas = within(canvasElement)
		await expect(canvas.getByRole('heading', { name: text.editor.title })).toBeVisible()
		await expect(canvas.getByRole('textbox', { name: text.editor.label })).toBeVisible()
		await expect(canvas.getByRole('button', { name: codeBlockAction?.label ?? 'Code block' })).toBeVisible()
		await expect(canvas.getByText(text.editor.previewTitle)).toBeVisible()
	}
}
