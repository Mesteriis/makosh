import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { expect, within } from 'storybook/test'
import {
	Accordion,
	ActionCard,
	Button,
	Callout,
	Card,
	CardContent,
	CardDescription,
	CardFooter,
	CardHeader,
	CardTitle,
	Divider,
	Fieldset,
	Input,
	Paper,
	Panel,
	Popover,
	Section,
	StatCard,
	Surface,
	ToolbarSection,
	Well
} from '@/shared/ui'
import { storybookLocaleFromGlobals } from './storybook-i18n'
import { generalStoryCopy } from './general-story-copy'

const meta = {
	title: 'Макошь UI/General/Surface',
	component: Surface
} satisfies Meta<typeof Surface>

export default meta
type Story = StoryObj<typeof meta>

export const Overview: Story = {
	render: (_args, context) => ({
		components: {
			Divider,
			Paper,
			Panel,
			Surface
		},
		data() {
			return { copy: generalStoryCopy(storybookLocaleFromGlobals(context.globals)) }
		},
		template: `
			<section class="storybook-canvas">
				<div class="storybook-section">
					<h2>{{ copy.surfaces.overviewTitle }}</h2>
					<p>{{ copy.surfaces.overviewDescription }}</p>
				</div>

				<Surface tone="default" class="storybook-section">
					<h3>{{ copy.surfaces.surfaceTitle }}</h3>
					<p>{{ copy.surfaces.labels.default }}</p>
					<Divider />
					<p>{{ copy.surfaces.labels.comfortable }}</p>
				</Surface>

				<div class="storybook-grid">
					<Paper tone="raised" class="storybook-section">
						<h3>{{ copy.surfaces.paperTitle }}</h3>
						<p>{{ copy.surfaces.labels.raised }}</p>
					</Paper>

					<Panel tone="muted" class="storybook-section">
						<h3>{{ copy.surfaces.panelTitle }}</h3>
						<p>{{ copy.surfaces.labels.muted }}</p>
					</Panel>
				</div>
			</section>
		`
	}),
	play: async ({ canvasElement, globals }) => {
		const copy = generalStoryCopy(storybookLocaleFromGlobals(globals))
		const canvas = within(canvasElement)

		await expect(canvas.getByText(copy.surfaces.overviewTitle)).toBeVisible()
		await expect(canvas.getByText(copy.surfaces.surfaceTitle)).toBeVisible()
	}
}

export const Cards: Story = {
	render: (_args, context) => ({
		components: {
			ActionCard,
			Card,
			CardContent,
			CardDescription,
			CardFooter,
			CardHeader,
			CardTitle,
			StatCard
		},
		data() {
			const copy = generalStoryCopy(storybookLocaleFromGlobals(context.globals))
			return {
				copy,
				stats: copy.surfaces.stats,
				actionCards: copy.surfaces.actionCards,
				signalCards: copy.surfaces.signalCards
			}
		},
		template: `
			<section class="storybook-canvas">
				<div class="storybook-section">
					<h2>{{ copy.surfaces.cardsTitle }}</h2>
					<p>{{ copy.surfaces.overviewDescription }}</p>
				</div>

					<Card>
						<CardHeader>
							<CardTitle>{{ copy.surfaces.labels.comfortable }}</CardTitle>
							<CardDescription>{{ copy.surfaces.labels.comfortable }}</CardDescription>
						</CardHeader>
					<CardContent>
						<div class="storybook-grid">
							<StatCard
								v-for="stat in stats"
								:key="stat.label"
								:description="stat.description"
								:icon="stat.icon"
								:label="stat.label"
								:tone="stat.tone"
								:trend="stat.trend"
								:value="stat.value"
							/>
						</div>
					</CardContent>
					<CardFooter>
						<span>{{ copy.surfaces.actions.primary }}</span>
					</CardFooter>
				</Card>

				<div class="storybook-section">
					<h3>{{ copy.surfaces.signalTitle }}</h3>
					<p>{{ copy.surfaces.signalDescription }}</p>
				</div>

				<div class="storybook-grid">
					<Card
						v-for="signalCard in signalCards"
						:key="signalCard.title"
						variant="raised"
						:signal="signalCard.active"
						:signal-tone="signalCard.tone"
						:signal-pulse="signalCard.pulse !== false"
					>
						<CardHeader>
							<CardTitle>{{ signalCard.title }}</CardTitle>
							<CardDescription>{{ signalCard.description }}</CardDescription>
						</CardHeader>
					</Card>
				</div>

				<div class="storybook-grid">
					<ActionCard
						v-for="(actionCard, index) in actionCards"
						:key="actionCard.title"
						:description="actionCard.description"
						:disabled="index === 2"
						:icon="actionCard.icon"
						:selected="index === 0"
						:title="actionCard.title"
					/>
				</div>
			</section>
		`
	}),
	play: async ({ canvasElement, globals }) => {
		const copy = generalStoryCopy(storybookLocaleFromGlobals(globals))
		const canvas = within(canvasElement)

		await expect(canvas.getByText(copy.surfaces.cardsTitle)).toBeVisible()
		await expect(canvas.getByText(copy.surfaces.signalCards[0].title)).toBeVisible()
		await expect(canvas.getByText(copy.surfaces.actionCards[0].title)).toBeVisible()
	}
}

export const Sections: Story = {
	render: (_args, context) => ({
		components: {
			Button,
			Divider,
			Section,
			Well
		},
		data() {
			return { copy: generalStoryCopy(storybookLocaleFromGlobals(context.globals)) }
		},
		template: `
			<section class="storybook-canvas">
				<Section tone="bordered">
					<template #header>
						<h2>{{ copy.surfaces.sectionsTitle }}</h2>
						<p>{{ copy.surfaces.overviewDescription }}</p>
					</template>
					<template #actions>
						<Button size="sm" variant="outline">{{ copy.surfaces.actions.secondary }}</Button>
					</template>

					<div class="storybook-grid">
						<Well tone="default">
							<strong>{{ copy.surfaces.labels.default }}</strong>
							<p>{{ copy.surfaces.labels.preview }}</p>
						</Well>
						<Well tone="muted">
							<strong>{{ copy.surfaces.labels.muted }}</strong>
							<p>{{ copy.surfaces.labels.details }}</p>
						</Well>
						<Well tone="inset">
							<strong>{{ copy.surfaces.labels.deep }}</strong>
							<p>{{ copy.surfaces.labels.compact }}</p>
						</Well>
					</div>

					<template #footer>
						<Divider />
						<span>{{ copy.surfaces.actions.save }}</span>
					</template>
				</Section>
			</section>
		`
	}),
	play: async ({ canvasElement, globals }) => {
		const copy = generalStoryCopy(storybookLocaleFromGlobals(globals))
		const canvas = within(canvasElement)

		await expect(canvas.getByText(copy.surfaces.sectionsTitle)).toBeVisible()
		await expect(canvas.getByText(copy.surfaces.actions.save)).toBeVisible()
	}
}

const AccordionStory: Story = {
	render: (_args, context) => ({
		components: {
			Accordion,
			Section
		},
		data() {
			const copy = generalStoryCopy(storybookLocaleFromGlobals(context.globals))
			return {
				copy,
				openItems: [copy.surfaces.accordionItems[0]?.id ?? 'density']
			}
		},
		template: `
			<section class="storybook-canvas">
				<Section tone="bordered">
					<template #header>
						<h2>{{ copy.surfaces.accordionTitle }}</h2>
						<p>{{ copy.surfaces.overviewDescription }}</p>
					</template>

					<Accordion v-model="openItems" :items="copy.surfaces.accordionItems">
						<template #item="{ item }">
							<p>{{ item.description }}</p>
						</template>
					</Accordion>
				</Section>
			</section>
		`
	}),
	play: async ({ canvasElement, globals }) => {
		const copy = generalStoryCopy(storybookLocaleFromGlobals(globals))
		const canvas = within(canvasElement)

		await expect(canvas.getByText(copy.surfaces.accordionTitle)).toBeVisible()
		await expect(canvas.getByText(copy.surfaces.accordionItems[0].title)).toBeVisible()
	}
}

export const CalloutsAndWells: Story = {
	render: (_args, context) => ({
		components: {
			Button,
			Callout,
			Well
		},
		data() {
			const copy = generalStoryCopy(storybookLocaleFromGlobals(context.globals))
			return { copy, callouts: copy.surfaces.callouts }
		},
		template: `
			<section class="storybook-canvas">
				<div class="storybook-section">
					<h2>{{ copy.surfaces.calloutsTitle }}</h2>
					<p>{{ copy.surfaces.overviewDescription }}</p>
				</div>

				<div class="storybook-stack">
					<Callout v-for="callout in callouts" :key="callout.title" :tone="callout.tone">
						<template #title>{{ callout.title }}</template>
						{{ callout.body }}
						<template #actions>
							<Button size="sm" variant="ghost">{{ copy.surfaces.actions.secondary }}</Button>
						</template>
					</Callout>
				</div>

				<Well tone="inset" padding="lg">
					<strong>{{ copy.surfaces.labels.details }}</strong>
					<p>{{ copy.surfaces.labels.preview }}</p>
				</Well>
			</section>
		`
	}),
	play: async ({ canvasElement, globals }) => {
		const copy = generalStoryCopy(storybookLocaleFromGlobals(globals))
		const canvas = within(canvasElement)

		await expect(canvas.getByText(copy.surfaces.calloutsTitle)).toBeVisible()
		await expect(canvas.getByText(copy.surfaces.callouts[0].title)).toBeVisible()
	}
}

export const FieldsetAndToolbar: Story = {
	render: (_args, context) => ({
			components: {
				Button,
				Fieldset,
				Input,
				ToolbarSection
			},
		data() {
			const copy = generalStoryCopy(storybookLocaleFromGlobals(context.globals))
			return { copy, fields: copy.surfaces.fields }
		},
		template: `
			<section class="storybook-canvas">
				<div class="storybook-section">
					<h2>{{ copy.surfaces.fieldsetTitle }}</h2>
					<p>{{ copy.surfaces.overviewDescription }}</p>
				</div>

					<Fieldset>
						<template #legend>{{ copy.surfaces.labels.details }}</template>
						<template #description>{{ copy.surfaces.labels.details }}</template>

						<div class="storybook-grid">
							<label v-for="field in fields" :key="field.label" class="storybook-stack">
								<strong>{{ field.label }}</strong>
								<Input :aria-label="field.label" :model-value="field.value" readonly />
							</label>
						</div>
				</Fieldset>

				<ToolbarSection>
					<template #label>{{ copy.surfaces.labels.toolbar }}</template>
					<Button size="sm">{{ copy.surfaces.actions.save }}</Button>
					<Button size="sm" variant="outline">{{ copy.surfaces.actions.reset }}</Button>
				</ToolbarSection>
			</section>
		`
	}),
	play: async ({ canvasElement, globals }) => {
		const copy = generalStoryCopy(storybookLocaleFromGlobals(globals))
		const canvas = within(canvasElement)

		await expect(canvas.getByText(copy.surfaces.fieldsetTitle)).toBeVisible()
		await expect(canvas.getByText(copy.surfaces.labels.toolbar)).toBeVisible()
	}
}

export const OverlaySafety: Story = {
	render: (_args, context) => ({
		components: {
			Button,
			Card,
			CardContent,
			CardDescription,
			CardHeader,
			CardTitle,
			Popover
		},
		data() {
			return {
				copy: generalStoryCopy(storybookLocaleFromGlobals(context.globals)),
				open: true
			}
		},
		template: `
				<section class="storybook-canvas">
					<div class="storybook-section">
						<h2>{{ copy.surfaces.overlayTitle }}</h2>
						<p>{{ copy.surfaces.overviewDescription }}</p>
					</div>

					<Card>
						<CardHeader>
							<CardTitle>{{ copy.surfaces.labels.preview }}</CardTitle>
							<CardDescription>{{ copy.surfaces.overviewDescription }}</CardDescription>
						</CardHeader>
					<CardContent>
						<Popover v-model:open="open" align="start" :close-label="copy.actions.close">
							<template #trigger>
								<Button variant="outline" icon="tabler:window">{{ copy.surfaces.labels.trigger }}</Button>
								</template>
								<div class="storybook-section">
									<h3>{{ copy.surfaces.labels.details }}</h3>
									<p>{{ copy.overlay.popoverBody }}</p>
								</div>
						</Popover>
					</CardContent>
				</Card>
			</section>
		`
	}),
	play: async ({ canvasElement, globals }) => {
		const copy = generalStoryCopy(storybookLocaleFromGlobals(globals))
		const canvas = within(canvasElement)
		const body = within(document.body)

		await expect(canvas.getByText(copy.surfaces.overlayTitle)).toBeVisible()
		await expect(body.getByText(copy.overlay.popoverBody)).toBeVisible()
	}
}

export { AccordionStory as Accordion }
