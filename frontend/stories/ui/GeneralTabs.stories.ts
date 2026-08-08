import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { TabContent, Tabs } from '@/shared/ui'
import { storybookLocaleFromGlobals } from './storybook-i18n'
import { generalStoryCopy } from './general-story-copy'

const meta = {
	title: 'Макошь UI/General/Tabs',
	component: Tabs,
	render: (_args, context) => ({
		components: { TabContent, Tabs },
		data() {
			const copy = generalStoryCopy(storybookLocaleFromGlobals(context.globals))
			return { copy, active: 'review' }
		},
		template: `
			<section class="storybook-canvas">
				<div class="storybook-section storybook-narrow">
					<h2>{{ copy.controls.tabs }}</h2>
					<Tabs v-model="active" :tabs="copy.tabs">
						<TabContent value="review">{{ copy.tabContent.review }}</TabContent>
						<TabContent value="evidence">{{ copy.tabContent.evidence }}</TabContent>
						<TabContent value="memory">{{ copy.tabContent.memory }}</TabContent>
					</Tabs>
				</div>
			</section>
		`
	})
} satisfies Meta<typeof Tabs>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {}
