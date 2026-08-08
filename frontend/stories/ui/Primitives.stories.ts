import type { Meta, StoryObj } from '@storybook/vue3-vite'
import {
	Chip,
	Container,
	Divider,
	Heading,
	Label,
	LinkButton,
	Panel,
	Paper,
	Paragraph,
	Tag,
	Text,
	TextButton
} from '@/shared/ui'
import { storybookLocaleFromGlobals, storybookText } from './storybook-i18n'

const meta = {
	title: 'Макошь UI/Foundation/Primitives',
	render: (_args, context) => ({
		components: {
			Chip,
			Container,
			Divider,
			Heading,
			Label,
			LinkButton,
			Panel,
			Paper,
			Paragraph,
			Tag,
			Text,
			TextButton
		},
		setup() {
			return { text: storybookText(storybookLocaleFromGlobals(context.globals)) }
		},
		template: `
			<section class="storybook-canvas">
				<Container size="wide">
					<div class="storybook-grid">
						<Panel tone="raised">
							<Heading :level="2">{{ text.primitives.typographyTitle }}</Heading>
							<Paragraph tone="muted">{{ text.primitives.typographyDescription }}</Paragraph>
							<Divider />
							<div class="storybook-stack">
								<Heading :level="3">{{ text.primitives.heading }}</Heading>
								<Paragraph>{{ text.primitives.paragraph }}</Paragraph>
								<Text tone="muted">{{ text.primitives.muted }}</Text>
								<Text tone="strong" weight="bold">{{ text.primitives.strong }}</Text>
								<Text tone="accent" weight="semibold">{{ text.primitives.accent }}</Text>
								<Label>{{ text.common.evidence }}</Label>
							</div>
						</Panel>

						<Panel>
							<Heading :level="2">{{ text.primitives.chipsTitle }}</Heading>
							<Paragraph tone="muted">{{ text.primitives.chipsDescription }}</Paragraph>
							<div class="storybook-row">
								<Chip icon="tabler:circle-dot">{{ text.primitives.chips[0] }}</Chip>
								<Chip tone="success" icon="tabler:check">{{ text.primitives.chips[1] }}</Chip>
								<Tag tone="warning" icon="tabler:alert-triangle">{{ text.primitives.chips[2] }}</Tag>
								<Tag tone="danger" icon="tabler:shield-exclamation">{{ text.primitives.chips[3] }}</Tag>
							</div>
						</Panel>
					</div>

					<Paper tone="raised" class="storybook-section">
						<Heading :level="2">{{ text.primitives.actionsTitle }}</Heading>
						<Paragraph tone="muted">{{ text.primitives.actionsDescription }}</Paragraph>
						<div class="storybook-row">
							<LinkButton href="#" icon="tabler:external-link">{{ text.primitives.openDocs }}</LinkButton>
							<TextButton tone="quiet" icon="tabler:dots">{{ text.primitives.quietAction }}</TextButton>
							<TextButton tone="danger" icon="tabler:trash">{{ text.primitives.dangerAction }}</TextButton>
						</div>
					</Paper>

					<Panel tone="muted" class="storybook-section">
						<Heading :level="2">{{ text.primitives.surfacesTitle }}</Heading>
						<Paragraph tone="soft">{{ text.primitives.surfacesDescription }}</Paragraph>
					</Panel>
				</Container>
			</section>
		`
	})
} satisfies Meta

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {}
