import type { Meta, StoryObj } from '@storybook/vue3-vite'
import {
	Breadcrumbs,
	Button,
	CommandPalette,
	ContextMenu,
	Menubar,
	Menu,
	NavigationMenu,
	Pagination,
	SearchPalette,
	TabContent,
	Tabs,
	Tree,
	TreeItem
} from '@/shared/ui'
import type { CommandGroup, NavigationItem, TreeItemData } from '@/shared/ui'
import { storybookLocaleFromGlobals, storybookText } from './storybook-i18n'

const meta = {
	title: 'Макошь UI/General/Navigation',
	render: (_args, context) => ({
		components: {
			Breadcrumbs,
			Button,
			CommandPalette,
			ContextMenu,
			Menubar,
			Menu,
			NavigationMenu,
			Pagination,
			SearchPalette,
			TabContent,
			Tabs,
			Tree,
			TreeItem
		},
		data() {
			const text = storybookText(storybookLocaleFromGlobals(context.globals))
			return {
				text,
				page: 4,
				activeTab: 'review',
				activeNav: 'radar',
				activeMenu: 'review',
				selectedTreeItem: 'review-queue',
				expandedTreeItems: ['workspace', 'signals'],
				commandOpen: false,
				searchOpen: false,
				breadcrumbItems: text.navigation.breadcrumbs as NavigationItem[],
				navItems: text.navigation.navItems as NavigationItem[],
				menuItems: text.navigation.menuItems as NavigationItem[],
				contextItems: text.navigation.contextItems as NavigationItem[],
				menubarItems: text.navigation.menubarItems as NavigationItem[],
				treeItems: text.navigation.treeItems as TreeItemData[],
				commandGroups: text.command.groups as CommandGroup[]
			}
		},
		template: `
			<section class="storybook-canvas">
				<div class="storybook-section">
					<h2>{{ text.navigation.title }}</h2>
					<p>{{ text.navigation.description }}</p>
					<Breadcrumbs :items="breadcrumbItems" :label="text.navigation.breadcrumbLabel" />
				</div>

				<div class="storybook-grid">
					<div class="storybook-section">
						<h3>{{ text.navigation.productNavigation }}</h3>
						<NavigationMenu v-model="activeNav" :items="navItems" :label="text.navigation.productNavigation" />
						<Tabs v-model="activeTab" :tabs="text.navigation.tabs">
							<TabContent value="review">{{ text.navigation.reviewContent }}</TabContent>
							<TabContent value="evidence">{{ text.navigation.evidenceContent }}</TabContent>
							<TabContent value="memory">{{ text.navigation.memoryContent }}</TabContent>
						</Tabs>
						<Pagination v-model="page" :page-count="12" :label="text.navigation.pagination" />
					</div>

					<div class="storybook-section">
						<h3>{{ text.navigation.localMenus }}</h3>
						<div class="storybook-row">
							<Menu v-model="activeMenu" :items="menuItems" :label="text.navigation.localMenus" />
							<ContextMenu :items="contextItems" :default-open="true" :label="text.navigation.contextMenu">
								<template #trigger>
									<Button variant="outline" icon="tabler:cursor-text">{{ text.navigation.contextTrigger }}</Button>
								</template>
							</ContextMenu>
						</div>
						<Menubar :items="menubarItems" :label="text.navigation.menubar" />
					</div>
				</div>

				<div class="storybook-grid">
					<div class="storybook-section">
						<h3>{{ text.navigation.tree }}</h3>
						<Tree
							v-model="selectedTreeItem"
							v-model:expanded="expandedTreeItems"
							:items="treeItems"
							:label="text.navigation.tree"
						/>
					</div>

					<div class="storybook-section">
						<h3>{{ text.navigation.palettes }}</h3>
						<div class="storybook-row">
							<CommandPalette
								v-model:open="commandOpen"
								:groups="commandGroups"
								:trigger-label="text.navigation.commandPalette"
								:placeholder="text.navigation.commandPlaceholder"
							/>
							<SearchPalette
								v-model:open="searchOpen"
								:groups="commandGroups"
								:trigger-label="text.navigation.searchPalette"
								:placeholder="text.navigation.searchPlaceholder"
							/>
						</div>
					</div>
				</div>
			</section>
		`
	})
} satisfies Meta

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {}
