import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { expect, within } from 'storybook/test'
import {
	ActivityItem,
	Avatar,
	Button,
	Card,
	CardContent,
	CardFooter,
	ComingSoonState,
	Counter,
	DescriptionList,
	EmptyState,
	ErrorState,
	KeyValue,
	List,
	LoadingState,
	Metric,
	NoDataState,
	NoSearchResultsState,
	OfflineState,
	PropertyGrid,
	Progress,
	Statistic,
	Skeleton,
	SkeletonPanel,
	TabContent,
	TabTrigger,
	Tabs,
	Table,
	TimelineItem,
	VirtualList,
	VirtualTable
} from '@/shared/ui'
import { storybookLocaleFromGlobals, storybookText } from './storybook-i18n'

const meta = {
	title: 'Макошь UI/General/Data Display'
} satisfies Meta

export default meta
type Story = StoryObj<typeof meta>

export const TablesAndLists: Story = {
	render: (_args, context) => ({
		components: {
			Avatar,
			Card,
			CardContent,
			CardFooter,
			List,
			Progress,
			Skeleton,
			SkeletonPanel,
			TabContent,
			TabTrigger,
			Tabs,
			Table,
			VirtualList,
			VirtualTable
		},
		data() {
			return { text: storybookText(storybookLocaleFromGlobals(context.globals)) }
		},
		template: `
			<section class="storybook-canvas">
				<div class="storybook-section">
					<h2>{{ text.data.title }}</h2>
					<p>{{ text.data.description }}</p>
				</div>

				<div class="storybook-grid">
					<div class="storybook-section">
						<h3>{{ text.data.tableTitle }}</h3>
						<Table
							:columns="text.data.tableColumns"
							:rows="text.data.tableRows"
							:caption="text.data.tableCaption"
						/>
					</div>
					<div class="storybook-section">
						<h3>{{ text.data.virtualTableTitle }}</h3>
						<VirtualTable
							:columns="text.data.tableColumns"
							:rows="text.data.tableRows"
							:caption="text.data.virtualTableTitle"
							:visible-count="3"
						/>
					</div>
				</div>

				<div class="storybook-grid">
					<div class="storybook-section">
						<h3>{{ text.data.listTitle }}</h3>
						<List :items="text.data.listItems" :label="text.data.listTitle" />
					</div>
					<div class="storybook-section">
						<h3>{{ text.data.virtualListTitle }}</h3>
						<VirtualList :items="text.data.listItems" :label="text.data.virtualListTitle" :visible-count="2" />
					</div>
				</div>

				<Card>
					<CardContent>
						<div class="storybook-row">
							<Avatar fallback="HH" alt="Макошь" />
							<Progress :model-value="64" />
							<Skeleton width="40%" height="12px" />
						</div>
						<SkeletonPanel
							title="Reserved projection"
							description="Full-size skeleton container for surfaces that are not wired yet."
							:rows="3"
						/>
						<Tabs default-value="display">
							<template #list>
								<TabTrigger value="display">{{ text.data.tableTitle }}</TabTrigger>
							</template>
							<TabContent value="display">{{ text.data.description }}</TabContent>
						</Tabs>
					</CardContent>
					<CardFooter>
						<Counter :value="text.data.counter.value" :label="text.data.counter.label" />
					</CardFooter>
				</Card>
			</section>
		`
	}),
	play: async ({ canvasElement, globals }) => {
		const text = storybookText(storybookLocaleFromGlobals(globals))
		const canvas = within(canvasElement)
		await expect(canvas.getByText(text.data.title)).toBeVisible()
		await expect(canvas.getByText(text.data.tableCaption)).toBeVisible()
	}
}

export const DetailsAndMetrics: Story = {
	render: (_args, context) => ({
		components: {
			ActivityItem,
			Counter,
			DescriptionList,
			KeyValue,
			Metric,
			PropertyGrid,
			Statistic,
			TimelineItem
		},
		data() {
			return { text: storybookText(storybookLocaleFromGlobals(context.globals)) }
		},
		template: `
			<section class="storybook-canvas">
				<div class="storybook-section">
					<h2>{{ text.data.detailsTitle }}</h2>
					<DescriptionList :items="text.data.properties" :title="text.data.detailsTitle" />
				</div>

				<div class="storybook-grid">
					<PropertyGrid :items="text.data.properties" columns="two" :title="text.data.propertyGridTitle" />
					<div class="storybook-section">
						<h3>{{ text.data.metricsTitle }}</h3>
						<div class="storybook-row">
							<Statistic
								:label="text.data.statistic.label"
								:value="text.data.statistic.value"
								:trend="text.data.statistic.trend"
								:description="text.data.statistic.description"
								:tone="text.data.statistic.tone"
							/>
							<Metric
								:label="text.data.metric.label"
								:value="text.data.metric.value"
								:unit="text.data.metric.unit"
								:delta="text.data.metric.delta"
								:tone="text.data.metric.tone"
							/>
						<Counter
							:value="text.data.counter.value"
							:max="text.data.counter.max"
							:label="text.data.counter.label"
							:tone="text.data.counter.tone"
						/>
						</div>
						<dl class="storybook-stack">
							<KeyValue
								:label="text.data.properties[0].label"
								:value="text.data.properties[0].value"
								:description="text.data.properties[0].description"
							/>
						</dl>
					</div>
				</div>

				<div class="storybook-grid">
					<div class="storybook-section">
						<h3>{{ text.data.timelineTitle }}</h3>
						<TimelineItem
							v-for="item in text.data.timelineItems"
							:key="item.id"
							:title="item.title"
							:description="item.description"
							:time="item.time"
							:icon="item.icon"
							:tone="item.tone"
						/>
					</div>
					<div class="storybook-section">
						<h3>{{ text.data.timelineTitle }}</h3>
						<ActivityItem
							v-for="item in text.data.activityItems"
							:key="item.id"
							:title="item.title"
							:description="item.description"
							:meta="item.meta"
							:icon="item.icon"
							:tone="item.tone"
						/>
					</div>
				</div>
			</section>
		`
	}),
	play: async ({ canvasElement, globals }) => {
		const text = storybookText(storybookLocaleFromGlobals(globals))
		const canvas = within(canvasElement)
		await expect(canvas.getAllByText(text.data.detailsTitle)[0]).toBeVisible()
		await expect(canvas.getByText(text.data.metric.label)).toBeVisible()
	}
}

export const States: Story = {
	render: (_args, context) => ({
		components: {
			Button,
			ComingSoonState,
			EmptyState,
			ErrorState,
			LoadingState,
			NoDataState,
			NoSearchResultsState,
			OfflineState
		},
		data() {
			return { text: storybookText(storybookLocaleFromGlobals(context.globals)) }
		},
		template: `
			<section class="storybook-canvas">
				<div class="storybook-section">
					<h2>{{ text.data.statesTitle }}</h2>
				</div>
				<div class="storybook-grid">
					<EmptyState :title="text.data.states.emptyTitle" :description="text.data.states.emptyDescription" />
					<LoadingState :title="text.data.states.loadingTitle" :description="text.data.states.loadingDescription" />
					<ErrorState :title="text.data.states.errorTitle" :description="text.data.states.errorDescription">
						<template #action>
							<Button size="sm" variant="outline">{{ text.common.review }}</Button>
						</template>
					</ErrorState>
					<NoDataState :title="text.data.states.noDataTitle" />
					<NoSearchResultsState
						:title="text.data.states.noSearchTitle"
						:description="text.data.states.noSearchDescription"
						:query="text.data.states.noSearchQuery"
					/>
					<OfflineState :title="text.data.states.offlineTitle" />
					<ComingSoonState :title="text.data.states.comingSoonTitle" />
				</div>
			</section>
		`
	}),
	play: async ({ canvasElement, globals }) => {
		const text = storybookText(storybookLocaleFromGlobals(globals))
		const canvas = within(canvasElement)
		await expect(canvas.getByText(text.data.statesTitle)).toBeVisible()
		await expect(canvas.getByText(text.data.states.errorTitle)).toBeVisible()
	}
}
