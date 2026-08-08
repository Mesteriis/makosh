import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { TimelineItem } from '@/shared/ui'
import { storybookLocaleFromGlobals } from './storybook-i18n'
import { generalStoryCopy } from './general-story-copy'

const meta = {
	title: 'Макошь UI/General/Timeline',
	component: TimelineItem,
	render: (_args, context) => ({
		components: { TimelineItem },
		setup() {
			return { copy: generalStoryCopy(storybookLocaleFromGlobals(context.globals)) }
		},
		template: `
			<section class="storybook-canvas">
				<div class="storybook-section storybook-narrow">
					<h2>{{ copy.controls.timeline }}</h2>
					<TimelineItem
						v-for="item in copy.data.timelineItems"
						:key="item.title"
						:description="item.description"
						:icon="item.icon"
						:time="item.time"
						:title="item.title"
						:tone="item.tone"
					/>
				</div>
			</section>
		`
	})
} satisfies Meta<typeof TimelineItem>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {}
