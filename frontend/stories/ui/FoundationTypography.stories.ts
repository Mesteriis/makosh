import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { Heading, Paragraph, Text } from '@/shared/ui'
import { storybookLocaleFromGlobals } from './storybook-i18n'
import { generalStoryCopy } from './general-story-copy'

const meta = {
	title: 'Макошь UI/Foundation/Typography',
	render: (_args, context) => ({
		components: { Heading, Paragraph, Text },
		setup() {
			return { copy: generalStoryCopy(storybookLocaleFromGlobals(context.globals)) }
		},
		template: `
			<section class="storybook-canvas">
				<div class="storybook-section">
					<Heading :level="2">{{ copy.foundation.heading }}</Heading>
					<Paragraph tone="muted">{{ copy.foundation.paragraph }}</Paragraph>
					<div class="storybook-stack">
						<Text size="xl" tone="strong" weight="bold">{{ copy.foundation.typography }}</Text>
						<Text size="md" tone="default">{{ copy.foundation.paragraph }}</Text>
						<Text size="sm" tone="muted">{{ copy.foundation.paragraph }}</Text>
					</div>
				</div>
			</section>
		`
	})
} satisfies Meta

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {}
