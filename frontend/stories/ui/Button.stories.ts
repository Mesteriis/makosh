import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { Button, IconButton, Kbd } from '@/shared/ui'
import { storybookLocaleFromGlobals, storybookText } from './storybook-i18n'

const meta = {
	title: 'Макошь UI/General/Button',
	component: Button,
	argTypes: {
		variant: { control: 'select', options: ['default', 'secondary', 'outline', 'ghost', 'destructive'] },
		size: { control: 'select', options: ['sm', 'md', 'lg'] },
		disabled: { control: 'boolean' },
		loading: { control: 'boolean' },
		icon: { control: 'text' }
	},
	args: {
		variant: 'default',
		size: 'md',
		disabled: false,
		loading: false,
		icon: 'tabler:sparkles'
	},
	render: (args, context) => ({
		components: { Button },
		setup() {
			return { args, text: storybookText(storybookLocaleFromGlobals(context.globals)) }
		},
		template: '<Button v-bind="args">{{ text.button.runAction }}</Button>'
	})
} satisfies Meta<typeof Button>

export default meta
type Story = StoryObj<typeof meta>

export const Playground: Story = {}

export const Variants: Story = {
	render: (_args, context) => ({
		components: { Button, IconButton, Kbd },
		setup() {
			return { text: storybookText(storybookLocaleFromGlobals(context.globals)) }
		},
		template: `
			<section class="storybook-canvas">
				<div class="storybook-section">
					<h2>{{ text.button.title }}</h2>
					<p>{{ text.button.description }}</p>
					<div class="storybook-row">
						<Button icon="tabler:bolt">{{ text.button.primary }}</Button>
						<Button variant="secondary" icon="tabler:settings">{{ text.button.secondary }}</Button>
						<Button variant="outline">{{ text.button.outline }}</Button>
						<Button variant="ghost">{{ text.button.ghost }}</Button>
						<Button variant="destructive" icon="tabler:trash">{{ text.button.delete }}</Button>
					</div>
					<div class="storybook-row">
						<Button size="sm">{{ text.button.small }}</Button>
						<Button size="md">{{ text.button.medium }}</Button>
						<Button size="lg">{{ text.button.large }}</Button>
						<Button loading>{{ text.button.loading }}</Button>
						<IconButton icon="tabler:search" :label="text.button.searchLabel" variant="outline" />
						<Kbd>⌘K</Kbd>
					</div>
				</div>
			</section>
		`
	})
}
