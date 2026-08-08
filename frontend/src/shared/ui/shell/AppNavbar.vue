<script setup lang="ts">
import { computed, onBeforeUnmount, ref, watch } from 'vue'
import Drawer from '../Drawer.vue'
import Icon from '../Icon.vue'
import Tree from '../Tree.vue'
import ToggleGroup from '../ToggleGroup.vue'
import { makoshBrandAssets } from '../assets/brand'
import type { UiThemeFamily, UiThemeMode } from '../foundation/theme'
import { useMouseLeaveDismiss } from '../useMouseLeaveDismiss'
import {
	buildAppNavbarHealthTree,
	healthStatusSignature,
	problemHealthGroupIds,
} from './appNavbarHealthTree'
import AppNavbarRouteBreadcrumbs from './AppNavbarRouteBreadcrumbs.vue'

import type { AppNavbarHealthCheck, AppNavbarHealthStatus } from './appNavbarHealthTree'

type AppNavbarBreadcrumb = {
	id: string
	label: string
}

type AppNavbarNavigationIconTone =
	| 'accounts'
	| 'calendar'
	| 'channels'
	| 'communication'
	| 'dashboard'
	| 'documents'
	| 'knowledge'
	| 'mail'
	| 'review'
	| 'settings'
	| 'tasks'
	| 'telegram'
	| 'whatsapp'
	| 'zulip'

type AppNavbarNavigationItem = AppNavbarBreadcrumb & {
	icon?: string
	iconTone?: AppNavbarNavigationIconTone
	disabled?: boolean
	disabledReason?: string
	loading?: boolean
}

type AppNavbarNavigationLevel = {
	id: string
	label: string
	currentItem: AppNavbarNavigationItem
	items: readonly AppNavbarNavigationItem[]
}

type AppNavbarToggleOption = {
	value: string
	label: string
}

type AppNavbarThemeFamilyOption = {
	value: UiThemeFamily
	label: string
}

type AppNavbarThemeModeOption = {
	value: UiThemeMode
	label: string
}

type AppNavbarNotification = {
	id: string
	title: string
	body?: string
	sourceLabel: string
	timeLabel: string
	icon: string
	tone: 'info' | 'success' | 'warning' | 'danger'
	targetView?: string
	targetId?: string
}

const props = withDefaults(defineProps<{
	breadcrumbs?: readonly AppNavbarBreadcrumb[]
	healthChecks?: readonly AppNavbarHealthCheck[]
	currentLanguage?: string
	currentThemeFamily?: UiThemeFamily
	currentThemeMode?: UiThemeMode
	languageOptions?: AppNavbarToggleOption[]
	navigationLevels?: readonly AppNavbarNavigationLevel[]
	notifications?: readonly AppNavbarNotification[]
	themeFamilyOptions?: AppNavbarThemeFamilyOption[]
	themeModeOptions?: AppNavbarThemeModeOption[]
	notificationsCount?: number
	healthStatusLabelVisibleMs?: number
}>(), {
	breadcrumbs: () => [],
	healthChecks: () => [],
	currentLanguage: 'ru',
	currentThemeFamily: 'base',
	currentThemeMode: 'light',
	healthStatusLabelVisibleMs: 5000,
	languageOptions: () => [],
	navigationLevels: () => [],
	notifications: () => [],
	themeFamilyOptions: () => [],
	themeModeOptions: () => []
})

const emit = defineEmits<{
	navigationSelect: [itemId: string]
	notificationDismiss: [notificationId: string]
	notificationSelect: [notificationId: string]
	notificationsClear: []
	languageChange: [value: string]
	themeFamilyChange: [value: UiThemeFamily]
	themeModeChange: [value: UiThemeMode]
}>()

const isHealthMenuOpen = ref(false)
const expandedHealthGroups = ref<string[]>([])
const isHealthLabelHeldOpen = ref(false)
const isHealthLabelHovered = ref(false)
const isNotificationsDrawerOpen = ref(false)
const isThemeMenuOpen = ref(false)
const isUserMenuOpen = ref(false)
const healthDropdownRef = ref<HTMLElement | null>(null)
const healthMenuRef = ref<HTMLElement | null>(null)
const themePickerRef = ref<HTMLElement | null>(null)
const themeListRef = ref<HTMLElement | null>(null)
const userDropdownRef = ref<HTMLElement | null>(null)
const userMenuRef = ref<HTMLElement | null>(null)
const selectedLanguage = ref(props.currentLanguage)
const selectedThemeFamily = ref<UiThemeFamily>(props.currentThemeFamily)
const selectedThemeMode = ref<UiThemeMode>(props.currentThemeMode)
let healthLabelTimer: ReturnType<typeof setTimeout> | undefined
const {
	cancelMouseLeaveDismiss: cancelHealthMouseLeaveDismiss,
	scheduleMouseLeaveDismiss: scheduleHealthMouseLeaveDismiss
} = useMouseLeaveDismiss(closeHealthMenu, undefined, {
	isOpen: isHealthMenuOpen,
	getBoundaryElements: () => [healthDropdownRef.value, healthMenuRef.value]
})
const {
	cancelMouseLeaveDismiss: cancelThemeMouseLeaveDismiss,
	scheduleMouseLeaveDismiss: scheduleThemeMouseLeaveDismiss
} = useMouseLeaveDismiss(closeThemeMenu, undefined, {
	isOpen: isThemeMenuOpen,
	getBoundaryElements: () => [themePickerRef.value, themeListRef.value]
})
const {
	cancelMouseLeaveDismiss: cancelUserMouseLeaveDismiss,
	scheduleMouseLeaveDismiss: scheduleUserMouseLeaveDismiss
} = useMouseLeaveDismiss(closeUserMenu, undefined, {
	isOpen: isUserMenuOpen,
	getBoundaryElements: () => [userDropdownRef.value, userMenuRef.value]
})

const systemStatus = computed<AppNavbarHealthStatus>(() => {
	for (const check of props.healthChecks) {
		if (check.status === 'unhealthy') return 'unhealthy'
		if (check.status === 'degraded') return 'degraded'
	}

	return 'healthy'
})

const systemStatusLabel = computed(() => {
	if (systemStatus.value === 'unhealthy') return 'System unhealthy'
	if (systemStatus.value === 'degraded') return 'System degraded'

	return 'System healthy'
})

const healthTreeItems = computed(() => buildAppNavbarHealthTree(props.healthChecks))

watch(() => healthStatusSignature(props.healthChecks), () => {
	expandedHealthGroups.value = problemHealthGroupIds(healthTreeItems.value)
}, { immediate: true })

const resolvedHealthStatusLabelVisibleMs = computed(() => {
	if (!Number.isFinite(props.healthStatusLabelVisibleMs)) return 5000

	return Math.max(0, props.healthStatusLabelVisibleMs)
})

const isHealthLabelVisible = computed(() => {
	return isHealthLabelHovered.value || isHealthLabelHeldOpen.value || isHealthMenuOpen.value
})

const selectedThemeFamilyLabel = computed(() => {
	for (const option of props.themeFamilyOptions) {
		if (option.value === selectedThemeFamily.value) return option.label
	}

	return selectedThemeFamily.value
})

const visibleNotifications = computed(() => props.notifications)
const resolvedNotificationsCount = computed(() => props.notificationsCount ?? visibleNotifications.value.length)
const notificationDrawerDescription = computed(() => {
	if (resolvedNotificationsCount.value === 0) return 'Новых уведомлений нет'

	return formatNotificationsCount(resolvedNotificationsCount.value)
})

function formatNotificationsCount(count: number): string {
	const lastTwoDigits = count % 100
	const lastDigit = count % 10

	if (lastTwoDigits >= 11 && lastTwoDigits <= 14) return `${count} активных уведомлений`
	if (lastDigit === 1) return `${count} активное уведомление`
	if (lastDigit >= 2 && lastDigit <= 4) return `${count} активных уведомления`

	return `${count} активных уведомлений`
}

function isNotificationClickable(notification: AppNavbarNotification): boolean {
	return Boolean(notification.targetView || notification.targetId)
}

function closeMenus(): void {
	cancelHealthMouseLeaveDismiss()
	cancelThemeMouseLeaveDismiss()
	cancelUserMouseLeaveDismiss()
	isHealthMenuOpen.value = false
	isThemeMenuOpen.value = false
	isUserMenuOpen.value = false
}

function closeHealthMenu(): void {
	cancelHealthMouseLeaveDismiss()
	isHealthMenuOpen.value = false
}

function closeThemeMenu(): void {
	cancelThemeMouseLeaveDismiss()
	isThemeMenuOpen.value = false
}

function closeUserMenu(): void {
	cancelUserMouseLeaveDismiss()
	cancelThemeMouseLeaveDismiss()
	isUserMenuOpen.value = false
	isThemeMenuOpen.value = false
}

function toggleHealthMenu(): void {
	cancelHealthMouseLeaveDismiss()
	isHealthMenuOpen.value = !isHealthMenuOpen.value
	isUserMenuOpen.value = false
	isThemeMenuOpen.value = false
}

function clearHealthLabelTimer(): void {
	if (healthLabelTimer === undefined) return

	clearTimeout(healthLabelTimer)
	healthLabelTimer = undefined
}

function revealHealthLabelTemporarily(): void {
	clearHealthLabelTimer()

	const visibleMs = resolvedHealthStatusLabelVisibleMs.value
	if (visibleMs === 0) {
		isHealthLabelHeldOpen.value = false
		return
	}

	isHealthLabelHeldOpen.value = true
	healthLabelTimer = setTimeout(() => {
		isHealthLabelHeldOpen.value = false
		healthLabelTimer = undefined
	}, visibleMs)
}

function handleHealthPointerEnter(): void {
	cancelHealthMouseLeaveDismiss()
	isHealthLabelHovered.value = true
	revealHealthLabelTemporarily()
}

function handleHealthPointerLeave(): void {
	isHealthLabelHovered.value = false
	scheduleHealthMouseLeaveDismiss()
}

function toggleUserMenu(): void {
	cancelUserMouseLeaveDismiss()
	isUserMenuOpen.value = !isUserMenuOpen.value
	isHealthMenuOpen.value = false
	isThemeMenuOpen.value = false
}

function updateLanguage(value: string | string[]): void {
	if (Array.isArray(value)) return

	selectedLanguage.value = value
	emit('languageChange', value)
}

function updateThemeFamily(value: string): void {
	const option = props.themeFamilyOptions.find((item) => item.value === value)
	if (!option) return

	selectedThemeFamily.value = option.value
	closeThemeMenu()
	emit('themeFamilyChange', option.value)
}

function updateThemeMode(value: string | string[]): void {
	if (Array.isArray(value)) return

	const option = props.themeModeOptions.find((item) => item.value === value)
	if (!option) return

	selectedThemeMode.value = option.value
	emit('themeModeChange', option.value)
}

function toggleThemeMenu(): void {
	cancelThemeMouseLeaveDismiss()
	isThemeMenuOpen.value = !isThemeMenuOpen.value
}

function handleNotificationsOpenChange(value: boolean): void {
	isNotificationsDrawerOpen.value = value

	if (value) {
		closeMenus()
	}
}

function handleNavigationSelect(itemId: string): void {
	emit('navigationSelect', itemId)
}

function handleBrandClick(): void {
	closeMenus()
	emit('navigationSelect', 'dashboard')
}

function handleSettingsClick(): void {
	closeMenus()
	emit('navigationSelect', 'settings')
}

watch(systemStatus, () => {
	revealHealthLabelTemporarily()
})

watch(() => props.currentThemeFamily, (value) => {
	selectedThemeFamily.value = value
})

watch(() => props.currentThemeMode, (value) => {
	selectedThemeMode.value = value
})

watch(() => props.currentLanguage, (value) => {
	selectedLanguage.value = value
})

onBeforeUnmount(() => {
	clearHealthLabelTimer()
})
</script>

<template>
	<nav class="app-navbar" aria-label="Application navigation" @keydown.esc="closeMenus">
		<button
			type="button"
			class="app-navbar__brand"
			aria-label="Dashboard"
			title="Dashboard"
			@click="handleBrandClick"
		>
			<span class="app-navbar__brand-mark" aria-hidden="true">
				<img
					:src="makoshBrandAssets.logoMarkLight"
					alt=""
					class="app-navbar__brand-logo app-navbar__brand-logo--light"
				/>
				<img
					:src="makoshBrandAssets.logoMarkDark"
					alt=""
					class="app-navbar__brand-logo app-navbar__brand-logo--dark"
				/>
			</span>
		</button>

		<AppNavbarRouteBreadcrumbs
			:levels="navigationLevels"
			@select="handleNavigationSelect"
		/>

		<div class="app-navbar__page-actions">
			<slot name="page-actions" />
		</div>

		<div class="app-navbar__actions" aria-label="System actions">
			<div
				ref="healthDropdownRef"
				class="app-navbar__dropdown"
				@mouseenter="handleHealthPointerEnter"
				@mouseleave="handleHealthPointerLeave"
			>
				<button
					type="button"
					class="app-navbar__health-button"
					:class="[
						`app-navbar__health-button--${systemStatus}`,
						{ 'app-navbar__health-button--label-visible': isHealthLabelVisible }
					]"
					:aria-label="systemStatusLabel"
					:aria-expanded="isHealthMenuOpen"
					:title="systemStatusLabel"
					aria-haspopup="dialog"
					@click="toggleHealthMenu"
				>
					<Icon icon="tabler:heartbeat" size="18" class="app-navbar__button-icon" />
					<span class="app-navbar__health-label" :aria-hidden="!isHealthLabelVisible">
						{{ systemStatusLabel }}
					</span>
				</button>

				<div
					ref="healthMenuRef"
					v-if="isHealthMenuOpen"
					class="app-navbar__menu app-navbar__health-menu"
					role="dialog"
					aria-label="System health checks"
				>
					<Tree
						v-model:expanded="expandedHealthGroups"
						:items="healthTreeItems"
						label="System health topology"
						class="app-navbar__health-tree"
					/>
				</div>
			</div>

			<Drawer
				:open="isNotificationsDrawerOpen"
				title="Уведомления"
				:description="notificationDrawerDescription"
				close-label="Закрыть уведомления"
				content-class="app-navbar__notifications-drawer"
				side="right"
				size="compact"
				@update:open="handleNotificationsOpenChange"
			>
				<template #trigger>
					<button
						type="button"
						class="app-navbar__icon-button"
						:aria-expanded="isNotificationsDrawerOpen"
						aria-haspopup="dialog"
						aria-label="Уведомления"
						@click="closeMenus"
					>
						<Icon icon="tabler:bell" size="18" class="app-navbar__button-icon" />
						<span v-if="resolvedNotificationsCount > 0" class="app-navbar__badge">
							{{ resolvedNotificationsCount }}
						</span>
					</button>
				</template>

				<div class="app-navbar__notifications">
					<div v-if="visibleNotifications.length === 0" class="app-navbar__notifications-empty">
						<Icon icon="tabler:bell-check" size="28" class="app-navbar__notifications-empty-icon" />
						<span>Новых уведомлений нет</span>
					</div>

					<template v-else>
						<header class="app-navbar__notifications-header">
							<strong>{{ notificationDrawerDescription }}</strong>
							<button
								type="button"
								class="app-navbar__notifications-clear"
								@click="emit('notificationsClear')"
							>
								Очистить все
							</button>
						</header>

						<ol class="app-navbar__notifications-list" aria-label="Список уведомлений">
							<li
								v-for="notification in visibleNotifications"
								:key="notification.id"
								class="app-navbar__notification"
								:class="[
									`app-navbar__notification--${notification.tone}`,
									{ 'app-navbar__notification--clickable': isNotificationClickable(notification) }
								]"
							>
								<button
									type="button"
									class="app-navbar__notification-main"
									:disabled="!isNotificationClickable(notification)"
									@click="emit('notificationSelect', notification.id)"
								>
									<span class="app-navbar__notification-icon" aria-hidden="true">
										<Icon :icon="notification.icon" size="18" />
									</span>
									<span class="app-navbar__notification-copy">
										<span class="app-navbar__notification-meta">
											<span>{{ notification.sourceLabel }}</span>
											<span>{{ notification.timeLabel }}</span>
										</span>
										<span class="app-navbar__notification-title">{{ notification.title }}</span>
										<span v-if="notification.body" class="app-navbar__notification-body">
											{{ notification.body }}
										</span>
									</span>
								</button>
								<button
									type="button"
									class="app-navbar__notification-dismiss"
									:aria-label="`Очистить уведомление: ${notification.title}`"
									:title="`Очистить уведомление: ${notification.title}`"
									@click.stop="emit('notificationDismiss', notification.id)"
								>
									<Icon icon="tabler:x" size="16" />
								</button>
							</li>
						</ol>
					</template>
				</div>
			</Drawer>

			<div
				ref="userDropdownRef"
				class="app-navbar__dropdown"
				@mouseenter="cancelUserMouseLeaveDismiss"
				@mouseleave="scheduleUserMouseLeaveDismiss"
			>
				<button
					type="button"
					class="app-navbar__icon-button"
					:aria-expanded="isUserMenuOpen"
					aria-haspopup="menu"
					aria-label="Menu"
					@click="toggleUserMenu"
				>
					<Icon icon="tabler:menu-2" size="18" class="app-navbar__button-icon" />
				</button>

				<div
					ref="userMenuRef"
					v-if="isUserMenuOpen"
					class="app-navbar__menu app-navbar__user-menu"
					role="menu"
					aria-label="Application menu"
				>
					<div v-if="languageOptions.length > 0" class="app-navbar__toggle-row" role="none">
						<ToggleGroup
							class="app-navbar__language-toggle makosh-toggle-group--tabs"
							:model-value="selectedLanguage"
							:items="languageOptions"
							aria-label="Смена языка"
							@update:model-value="updateLanguage"
						/>
					</div>

					<div
						v-if="themeFamilyOptions.length > 0 && themeModeOptions.length > 0"
						class="app-navbar__select-row"
						role="none"
					>
						<span class="app-navbar__select-label">
							<Icon icon="tabler:sun-moon" size="18" class="app-navbar__menu-icon" />
							<span>Смена темы</span>
						</span>
						<div
							ref="themePickerRef"
							class="app-navbar__theme-picker"
							@mouseenter="cancelThemeMouseLeaveDismiss"
							@mouseleave="scheduleThemeMouseLeaveDismiss"
						>
							<button
								type="button"
								class="app-navbar__theme-trigger"
								:aria-expanded="isThemeMenuOpen"
								aria-haspopup="listbox"
								aria-label="Семейство темы"
								@click="toggleThemeMenu"
							>
								<span>{{ selectedThemeFamilyLabel }}</span>
								<Icon icon="tabler:chevron-down" size="16" class="app-navbar__theme-chevron" />
							</button>

							<div
								ref="themeListRef"
								v-if="isThemeMenuOpen"
								class="app-navbar__theme-list"
								role="listbox"
								aria-label="Семейство темы"
							>
								<button
									v-for="option in themeFamilyOptions"
									:key="option.value"
									type="button"
									class="app-navbar__theme-option"
									role="option"
									:aria-selected="selectedThemeFamily === option.value"
									@click="updateThemeFamily(option.value)"
								>
									<span>{{ option.label }}</span>
									<Icon
										v-if="selectedThemeFamily === option.value"
										icon="tabler:check"
										size="16"
										class="app-navbar__theme-check"
									/>
								</button>
							</div>
						</div>

						<ToggleGroup
							class="app-navbar__theme-mode-toggle makosh-toggle-group--tabs"
							:model-value="selectedThemeMode"
							:items="themeModeOptions"
							aria-label="Вариант темы"
							@update:model-value="updateThemeMode"
						/>
					</div>

					<button
						type="button"
						class="app-navbar__menu-item"
						role="menuitem"
						@click="handleSettingsClick"
					>
						<Icon icon="tabler:settings" size="18" class="app-navbar__menu-icon" />
						<span>Настройки</span>
					</button>

					<div class="app-navbar__menu-separator" role="separator" />

					<button
						type="button"
						class="app-navbar__menu-item app-navbar__menu-item--danger"
						role="menuitem"
					>
						<Icon icon="tabler:logout" size="18" class="app-navbar__menu-icon" />
						<span>Выход</span>
					</button>
				</div>
			</div>
		</div>
	</nav>
</template>
