import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { Icon, Kbd } from '@/shared/ui'
import { storybookLocaleFromGlobals } from './storybook-i18n'
import { generalStoryCopy } from './general-story-copy'

const icons = ['tabler:messages', 'tabler:radar', 'tabler:brain', 'tabler:archive', 'tabler:checks']

const meta = {
	title: 'Макошь UI/Foundation/Icons',
	component: Icon,
	render: (_args, context) => ({
		components: { Icon, Kbd },
		setup() {
			return { copy: generalStoryCopy(storybookLocaleFromGlobals(context.globals)), icons }
		},
		template: `
			<section class="storybook-canvas">
				<div class="storybook-section">
					<h2>{{ copy.foundation.icons }}</h2>
					<div class="storybook-row">
						<span v-for="icon in icons" :key="icon" class="storybook-row">
							<Icon :icon="icon" />
							<Kbd>{{ icon.replace('tabler:', '') }}</Kbd>
						</span>
					</div>
				</div>
			</section>
		`
	})
} satisfies Meta<typeof Icon>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {}
