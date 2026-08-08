import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { Button, Command } from '@/shared/ui'
import type { CommandGroup, CommandItem } from '@/shared/ui'
import { storybookLocaleFromGlobals, storybookText } from './storybook-i18n'

const meta = {
	title: 'Макошь UI/General/Command',
	component: Command,
	render: (_args, context) => ({
		components: { Button, Command },
		data() {
			const text = storybookText(storybookLocaleFromGlobals(context.globals))
			return {
				open: false,
				text,
				groups: text.command.groups as CommandGroup[],
				selected: null as CommandItem | null
			}
		},
		template: `
			<section class="storybook-canvas">
				<div class="storybook-section">
					<h2>{{ text.command.title }}</h2>
					<p>{{ text.command.description }}</p>
					<div class="storybook-row">
						<Button icon="tabler:command" @click="open = true">{{ text.command.open }}</Button>
						<span v-if="selected">{{ text.command.selected }}: {{ selected.label }}</span>
					</div>
				</div>
				<Command v-model:open="open" :groups="groups" @select="selected = $event" />
			</section>
		`
	})
} satisfies Meta<typeof Command>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {}
