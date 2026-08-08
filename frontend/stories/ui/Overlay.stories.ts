import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { expect, userEvent, waitFor, within } from 'storybook/test'
import {
	AlertDialog,
	Button,
	Dialog,
	Drawer,
	DropdownMenu,
	DropdownMenuItem,
	DropdownMenuLabel,
	DropdownMenuSeparator,
	FocusTrap,
	HoverCard,
	OverlayHost,
	Popover,
	Portal,
	Sheet,
	Tooltip
} from '@/shared/ui'
import { storybookLocaleFromGlobals, storybookText } from './storybook-i18n'

const meta = {
	title: 'Макошь UI/General/Overlays'
} satisfies Meta

export default meta
type Story = StoryObj<typeof meta>

export const PopoversAndMenus: Story = {
	render: (_args, context) => ({
		components: {
			Button,
			DropdownMenu,
			DropdownMenuItem,
			DropdownMenuLabel,
			DropdownMenuSeparator,
			HoverCard,
			Popover,
			Tooltip
		},
		data() {
			return {
				text: storybookText(storybookLocaleFromGlobals(context.globals)),
				popoverOpen: true,
				hoverCardOpen: true
			}
		},
		template: `
			<section class="storybook-canvas">
				<div class="storybook-section">
					<h2>{{ text.overlay.title }}</h2>
					<p>{{ text.overlay.description }}</p>
					<div class="storybook-row">
						<DropdownMenu>
							<template #trigger>
								<Button variant="outline" icon="tabler:menu-2">{{ text.overlay.menu }}</Button>
							</template>
							<DropdownMenuLabel>{{ text.overlay.navigation }}</DropdownMenuLabel>
							<DropdownMenuItem icon="tabler:messages">{{ text.overlay.communications }}</DropdownMenuItem>
							<DropdownMenuItem icon="tabler:radar">{{ text.overlay.radar }}</DropdownMenuItem>
							<DropdownMenuSeparator />
							<DropdownMenuItem icon="tabler:settings">{{ text.overlay.settings }}</DropdownMenuItem>
						</DropdownMenu>

						<Popover v-model:open="popoverOpen" align="start" :close-label="text.common.close">
							<template #trigger>
								<Button variant="ghost" icon="tabler:info-circle">{{ text.overlay.context }}</Button>
							</template>
							<div class="storybook-section">
								<h3>{{ text.overlay.popoverTitle }}</h3>
								<p>{{ text.overlay.popoverDescription }}</p>
							</div>
						</Popover>

						<HoverCard v-model:open="hoverCardOpen" align="start" :aria-label="text.overlay.hoverCardTitle">
							<template #trigger>
								<Button variant="outline" icon="tabler:archive">{{ text.overlay.hoverCardButton }}</Button>
							</template>
							<div class="storybook-stack">
								<strong>{{ text.overlay.hoverCardTitle }}</strong>
								<span>{{ text.overlay.hoverCardDescription }}</span>
							</div>
						</HoverCard>

						<Tooltip :content="text.overlay.tooltipContent">
							<template #trigger>
								<Button variant="outline" icon="tabler:help">{{ text.overlay.tooltipButton }}</Button>
							</template>
						</Tooltip>
					</div>
				</div>
			</section>
		`
	}),
	play: async ({ canvasElement, globals }) => {
		const text = storybookText(storybookLocaleFromGlobals(globals))
		const canvas = within(canvasElement)
		await expect(canvas.getByText(text.overlay.title)).toBeVisible()
		await expect(within(document.body).getByText(text.overlay.popoverTitle)).toBeVisible()
		await expect(within(document.body).getByText(text.overlay.hoverCardTitle)).toBeVisible()
	}
}

export const ModalSurfaces: Story = {
	render: (_args, context) => ({
		components: {
			AlertDialog,
			Button,
			Dialog,
			Drawer,
			Sheet
		},
		data() {
			return {
				text: storybookText(storybookLocaleFromGlobals(context.globals)),
				alertOpen: false,
				dialogOpen: false,
				drawerOpen: false,
				sheetOpen: false
			}
		},
		template: `
			<section class="storybook-canvas">
				<div class="storybook-section">
					<h2>{{ text.overlay.modalSurfacesTitle }}</h2>
					<p>{{ text.overlay.description }}</p>
					<div class="storybook-row">
						<Button icon="tabler:window" @click="dialogOpen = true">{{ text.overlay.openDialog }}</Button>
						<Button variant="secondary" icon="tabler:layout-sidebar-right" @click="sheetOpen = true">{{ text.overlay.openSheet }}</Button>
						<Button variant="outline" icon="tabler:layout-bottombar" @click="drawerOpen = true">{{ text.overlay.openDrawer }}</Button>
						<Button variant="destructive" icon="tabler:alert-triangle" @click="alertOpen = true">{{ text.overlay.openAlertDialog }}</Button>
					</div>
				</div>

				<Dialog v-model:open="dialogOpen" :title="text.overlay.dialogTitle" :description="text.overlay.dialogDescription">
					<p>{{ text.overlay.dialogBody }}</p>
					<template #footer>
						<Button variant="ghost" @click="dialogOpen = false">{{ text.common.cancel }}</Button>
						<Button @click="dialogOpen = false">{{ text.common.create }}</Button>
					</template>
				</Dialog>

				<Sheet v-model:open="sheetOpen" :title="text.overlay.sheetTitle" :description="text.overlay.sheetDescription">
					<p>{{ text.overlay.sheetBody }}</p>
					<template #footer>
						<Button @click="sheetOpen = false">{{ text.common.done }}</Button>
					</template>
				</Sheet>

				<Drawer v-model:open="drawerOpen" :title="text.overlay.drawerTitle" :description="text.overlay.drawerDescription">
					<p>{{ text.overlay.drawerBody }}</p>
					<template #footer>
						<Button @click="drawerOpen = false">{{ text.common.done }}</Button>
					</template>
				</Drawer>

				<AlertDialog
					v-model:open="alertOpen"
					:title="text.overlay.alertDialogTitle"
					:description="text.overlay.alertDialogDescription"
					:cancel-label="text.overlay.alertDialogCancel"
					:action-label="text.overlay.alertDialogAction"
				>
					<p>{{ text.overlay.alertDialogBody }}</p>
				</AlertDialog>
			</section>
		`
	}),
	play: async ({ canvasElement, globals }) => {
		const text = storybookText(storybookLocaleFromGlobals(globals))
		const canvas = within(canvasElement)
		const body = within(document.body)
		await expect(canvas.getByText(text.overlay.modalSurfacesTitle)).toBeVisible()

		await userEvent.click(canvas.getByText(text.overlay.openDialog))
		await waitFor(() => expect(body.getByRole('dialog', { name: text.overlay.dialogTitle })).toBeVisible())
		const dialog = body.getByRole('dialog', { name: text.overlay.dialogTitle })
		await userEvent.click(within(dialog).getByRole('button', { name: text.common.cancel }))
		await waitFor(() => {
			const dialogAfterClose = body.queryByRole('dialog', { name: text.overlay.dialogTitle })
			if (dialogAfterClose) expect(dialogAfterClose).not.toBeVisible()
		})

		await userEvent.click(canvas.getByText(text.overlay.openSheet))
		await waitFor(() => expect(body.getByRole('dialog', { name: text.overlay.sheetTitle })).toBeVisible())
		const sheet = body.getByRole('dialog', { name: text.overlay.sheetTitle })
		await userEvent.click(within(sheet).getByRole('button', { name: text.common.done }))
		await waitFor(() => {
			const sheetAfterClose = body.queryByRole('dialog', { name: text.overlay.sheetTitle })
			if (sheetAfterClose) expect(sheetAfterClose).not.toBeVisible()
		})

		await userEvent.click(canvas.getByText(text.overlay.openDrawer))
		await waitFor(() => expect(body.getByRole('dialog', { name: text.overlay.drawerTitle })).toBeVisible())
		const drawer = body.getByRole('dialog', { name: text.overlay.drawerTitle })
		await userEvent.click(within(drawer).getByRole('button', { name: text.common.done }))
		await waitFor(() => {
			const drawerAfterClose = body.queryByRole('dialog', { name: text.overlay.drawerTitle })
			if (drawerAfterClose) expect(drawerAfterClose).not.toBeVisible()
		})

		await userEvent.click(canvas.getByText(text.overlay.openAlertDialog))
		await waitFor(() => expect(body.getByRole('alertdialog', { name: text.overlay.alertDialogTitle })).toBeVisible())
		const alertDialog = body.getByRole('alertdialog', { name: text.overlay.alertDialogTitle })
		await userEvent.click(within(alertDialog).getByRole('button', { name: text.overlay.alertDialogCancel }))
		await waitFor(() => {
			const alertDialogAfterClose = body.queryByRole('alertdialog', { name: text.overlay.alertDialogTitle })
			if (alertDialogAfterClose) expect(alertDialogAfterClose).not.toBeVisible()
		})

		if (document.activeElement instanceof HTMLElement) {
			document.activeElement.blur()
		}
	}
}

export const Infrastructure: Story = {
	render: (_args, context) => ({
		components: {
			Button,
			FocusTrap,
			OverlayHost,
			Portal
		},
		data() {
			return {
				text: storybookText(storybookLocaleFromGlobals(context.globals))
			}
		},
		template: `
			<section class="storybook-canvas">
				<div class="storybook-section">
					<h2>{{ text.overlay.infrastructureTitle }}</h2>
					<p>{{ text.overlay.overlayHostDescription }}</p>
				</div>

				<OverlayHost passive layer="popover" :label="text.overlay.overlayHostTitle">
					<div class="storybook-section">
						<h3>{{ text.overlay.overlayHostTitle }}</h3>
						<p>{{ text.overlay.overlayHostDescription }}</p>
					</div>
				</OverlayHost>

				<Portal disabled>
					<div class="storybook-section">
						<h3>{{ text.overlay.portalTitle }}</h3>
						<p>{{ text.overlay.portalDescription }}</p>
					</div>
				</Portal>

				<FocusTrap :trapped="false" class="storybook-section">
					<h3>{{ text.overlay.focusTrapTitle }}</h3>
					<p>{{ text.overlay.focusTrapDescription }}</p>
					<div class="storybook-row">
						<Button size="sm" variant="outline">{{ text.common.cancel }}</Button>
						<Button size="sm">{{ text.common.done }}</Button>
					</div>
				</FocusTrap>
			</section>
		`
	}),
	play: async ({ canvasElement, globals }) => {
		const text = storybookText(storybookLocaleFromGlobals(globals))
		const canvas = within(canvasElement)
		await expect(canvas.getByText(text.overlay.infrastructureTitle)).toBeVisible()
		await expect(canvas.getByText(text.overlay.portalTitle)).toBeVisible()
		await expect(canvas.getByText(text.overlay.focusTrapTitle)).toBeVisible()
	}
}
