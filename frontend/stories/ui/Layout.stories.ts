import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { expect, within } from 'storybook/test'
import {
	ActionBar,
	Badge,
	BottomBar,
	Button,
	Dock,
	Flex,
	FloatingPanel,
	Grid,
	HStack,
	InspectorPanel,
	Resizable,
	ScrollArea,
	SidePanel,
	Split,
	Spacer,
	Stack,
	StatusBar,
	Toolbar,
	ToolbarGroup,
	ToolbarSeparator,
	TopBar,
	VStack,
	VirtualScrollArea
} from '@/shared/ui'
import { storybookLocaleFromGlobals, storybookText } from './storybook-i18n'

const meta = {
	title: 'Макошь UI/General/Layout'
} satisfies Meta

export default meta
type Story = StoryObj<typeof meta>

export const Composition: Story = {
	render: (_args, context) => ({
		components: {
			Badge,
			Flex,
			Grid,
			HStack,
			Resizable,
			Split,
			Stack,
			VStack
		},
		data() {
			return { text: storybookText(storybookLocaleFromGlobals(context.globals)) }
		},
		template: `
			<section class="storybook-canvas">
				<div class="storybook-section">
					<h2>{{ text.layout.compositionTitle }}</h2>
					<p>{{ text.layout.compositionDescription }}</p>
				</div>

				<Grid columns="auto" gap="lg">
					<VStack class="storybook-layout-card" gap="md">
						<HStack justify="between">
							<strong>{{ text.layout.stackTitle }}</strong>
							<Badge variant="accent">{{ text.common.evidence }}</Badge>
						</HStack>
						<Stack gap="sm">
							<span>{{ text.layout.stackDescription }}</span>
							<span>{{ text.data.states.noSearchDescription }}</span>
						</Stack>
					</VStack>

					<VStack class="storybook-layout-card" gap="md">
						<strong>{{ text.layout.gridTitle }}</strong>
						<Grid columns="three" gap="sm">
							<div v-for="card in text.layout.cards" :key="card.id" class="storybook-layout-tile">
								<strong>{{ card.meta }}</strong>
								<span>{{ card.title }}</span>
							</div>
						</Grid>
					</VStack>
				</Grid>

				<Split ratio="start" gap="lg" class="storybook-layout-card">
					<template #primary>
						<VStack gap="sm">
							<strong>{{ text.layout.splitPrimary }}</strong>
							<span>{{ text.layout.cards[0].description }}</span>
						</VStack>
					</template>
					<template #secondary>
						<VStack gap="sm">
							<strong>{{ text.layout.splitSecondary }}</strong>
							<span>{{ text.layout.cards[1].description }}</span>
						</VStack>
					</template>
				</Split>

				<Resizable axis="both" tone="raised" class="storybook-layout-resizable">
					<Flex direction="column" gap="sm">
						<strong>{{ text.layout.resizableTitle }}</strong>
						<span>{{ text.layout.cards[2].description }}</span>
					</Flex>
				</Resizable>

				<div class="storybook-layout-card" dir="rtl">
					<HStack justify="between" wrap>
						<span>{{ text.layout.stackTitle }}</span>
						<span>{{ text.common.openContext }}</span>
					</HStack>
				</div>
			</section>
		`
	}),
	play: async ({ canvasElement, globals }) => {
		const text = storybookText(storybookLocaleFromGlobals(globals))
		const canvas = within(canvasElement)
		await expect(canvas.getByText(text.layout.compositionTitle)).toBeVisible()
		await expect(canvas.getByText(text.layout.splitPrimary)).toBeVisible()
	}
}

export const ShellSurfaces: Story = {
	render: (_args, context) => ({
		components: {
			ActionBar,
			BottomBar,
			Button,
			Dock,
			HStack,
			InspectorPanel,
			SidePanel,
			Spacer,
			StatusBar,
			Toolbar,
			ToolbarGroup,
			ToolbarSeparator,
			TopBar,
			VStack
		},
		data() {
			return { text: storybookText(storybookLocaleFromGlobals(context.globals)) }
		},
		template: `
			<section class="storybook-canvas storybook-canvas--wide">
				<div class="storybook-section">
					<h2>{{ text.layout.shellTitle }}</h2>
					<p>{{ text.layout.shellDescription }}</p>
				</div>

				<div class="storybook-layout-shell">
					<TopBar :title="text.layout.topTitle" :description="text.layout.topDescription">
						<template #start>
							<Button size="sm" variant="ghost" icon="tabler:layout-sidebar" :aria-label="text.layout.dockLabel" />
						</template>
						<template #end>
							<Toolbar :label="text.layout.toolbarLabel" density="compact">
								<ToolbarGroup :label="text.layout.toolbarLabel">
									<Button size="sm" variant="ghost">{{ text.layout.toolbarActions[0] }}</Button>
									<Button size="sm" variant="ghost">{{ text.layout.toolbarActions[1] }}</Button>
								</ToolbarGroup>
								<Spacer orientation="horizontal" />
								<ToolbarSeparator />
								<ToolbarGroup :label="text.layout.actionLabel">
									<Button v-for="action in text.layout.toolbarActions.slice(2)" :key="action" size="sm" variant="ghost">
										{{ action }}
									</Button>
								</ToolbarGroup>
							</Toolbar>
						</template>
					</TopBar>

					<div class="storybook-layout-shell__body">
						<Dock :label="text.layout.dockLabel" compact>
							<Button v-for="item in text.layout.navItems" :key="item" size="sm" variant="ghost" icon="tabler:circle" :aria-label="item" />
						</Dock>
						<SidePanel :title="text.layout.sideTitle" width="compact">
							<VStack gap="sm">
								<Button v-for="item in text.layout.navItems" :key="item" size="sm" variant="ghost">
									{{ item }}
								</Button>
							</VStack>
						</SidePanel>
						<div class="storybook-layout-shell__main" role="region" :aria-label="text.layout.inspectorTitle">
							<InspectorPanel :title="text.layout.inspectorTitle" :description="text.layout.inspectorDescription">
								<VStack gap="sm">
									<HStack v-for="card in text.layout.cards" :key="card.id" justify="between">
										<span>{{ card.title }}</span>
										<strong>{{ card.meta }}</strong>
									</HStack>
								</VStack>
								<template #footer>
									<ActionBar :label="text.layout.actionLabel" justify="end">
										<Button size="sm" variant="ghost">{{ text.layout.actions[0] }}</Button>
										<Button size="sm">{{ text.layout.actions[1] }}</Button>
									</ActionBar>
								</template>
							</InspectorPanel>
						</div>
					</div>

					<BottomBar :label="text.layout.bottomLabel" density="compact">
						<StatusBar :items="text.layout.statusItems" :label="text.layout.statusLabel" />
					</BottomBar>
				</div>
			</section>
		`
	}),
	play: async ({ canvasElement, globals }) => {
		const text = storybookText(storybookLocaleFromGlobals(globals))
		const canvas = within(canvasElement)
		await expect(canvas.getByText(text.layout.shellTitle)).toBeVisible()
		await expect(canvas.getByText(text.layout.inspectorTitle)).toBeVisible()
	}
}

export const ScrollAndFloating: Story = {
	render: (_args, context) => ({
		components: {
			FloatingPanel,
			Grid,
			ScrollArea,
			VStack,
			VirtualScrollArea
		},
		data() {
			return { text: storybookText(storybookLocaleFromGlobals(context.globals)) }
		},
		template: `
			<section class="storybook-canvas">
				<div class="storybook-section">
					<h2>{{ text.layout.scrollTitle }}</h2>
					<p>{{ text.layout.scrollDescription }}</p>
				</div>

				<Grid columns="two" gap="lg">
					<VStack class="storybook-layout-card" gap="sm">
						<strong>{{ text.foundation.scrollTitle }}</strong>
						<ScrollArea size="md" class="storybook-layout-scroll">
							<div v-for="item in text.layout.virtualItems" :key="item" class="storybook-scroll-item">
								{{ item }}
							</div>
						</ScrollArea>
					</VStack>

					<VStack class="storybook-layout-card" gap="sm">
						<strong>{{ text.layout.virtualLabel }}</strong>
						<VirtualScrollArea :label="text.layout.virtualLabel" :total="text.layout.virtualItems.length" :visible-count="5">
							<template #default="{ visibleStart, visibleEnd }">
								<div
									v-for="item in text.layout.virtualItems.slice(visibleStart, visibleEnd)"
									:key="item"
									class="storybook-scroll-item"
								>
									{{ item }}
								</div>
							</template>
						</VirtualScrollArea>
					</VStack>
				</Grid>

				<FloatingPanel :label="text.layout.floatingTitle" class="storybook-layout-floating">
					<strong>{{ text.layout.floatingTitle }}</strong>
					<span>{{ text.layout.floatingDescription }}</span>
				</FloatingPanel>
			</section>
		`
	}),
	play: async ({ canvasElement, globals }) => {
		const text = storybookText(storybookLocaleFromGlobals(globals))
		const canvas = within(canvasElement)
		await expect(canvas.getByText(text.layout.scrollTitle)).toBeVisible()
		await expect(canvas.getByText(text.layout.floatingTitle)).toBeVisible()
	}
}
