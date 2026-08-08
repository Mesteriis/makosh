<script setup lang="ts">
import { computed } from 'vue'
import { Icon } from '@/shared/ui'
import type {
	MailFolderRow,
	MailOperationalReadModel,
	MailThreadRow,
} from './mailOperationalReadModel'
import { filterMailMessageRows } from './mailOperationalReadModel'

const props = defineProps<{
	model: MailOperationalReadModel
	searchQuery: string
}>()

const emit = defineEmits<{
	loadMore: []
	selectFolder: [folderId: string]
	selectMessage: [messageId: string]
	selectThread: [providerThreadId: string]
}>()

const visibleMessages = computed(() => filterMailMessageRows(props.model.messages, props.searchQuery))

function folderIcon(folder: MailFolderRow): string {
	const label = folder.label.toLocaleLowerCase()
	if (label.includes('inbox')) return 'tabler:inbox'
	if (label.includes('sent')) return 'tabler:send'
	if (label.includes('draft')) return 'tabler:file-pencil'
	if (label.includes('trash')) return 'tabler:trash'
	if (label.includes('spam')) return 'tabler:alert-octagon'
	if (label.includes('archive')) return 'tabler:archive'
	return 'tabler:folder'
}

function threadLabel(thread: MailThreadRow | undefined): string {
	return thread?.subject || 'Messages'
}
</script>

<template>
	<aside class="mail-workspace-list" aria-label="Mail list">
		<section class="mail-folder-strip" aria-label="Mail folders">
			<header>
				<strong>Folders</strong>
				<span>{{ model.folders.length }}</span>
			</header>
			<nav>
				<button
					v-for="folder in model.folders"
					:key="folder.id"
					type="button"
					:class="{ active: folder.selected }"
					:aria-pressed="folder.selected"
					@click="emit('selectFolder', folder.id)"
				>
					<Icon :icon="folderIcon(folder)" size="1rem" />
					<span>{{ folder.label }}</span>
					<small>{{ folder.meta.split(' · ')[0] }}</small>
				</button>
			</nav>
		</section>

		<section class="mail-list-view">
			<header class="mail-list-view__header">
				<div>
					<strong>{{ threadLabel(model.threads.find((thread) => thread.selected)) }}</strong>
					<small>{{ visibleMessages.length }} messages</small>
				</div>
				<select
					aria-label="Filter messages by thread"
					:value="model.threads.find((thread) => thread.selected)?.id || ''"
					@change="emit('selectThread', ($event.target as HTMLSelectElement).value)"
				>
					<option value="">All messages</option>
					<option v-for="thread in model.threads" :key="thread.id" :value="thread.id">
						{{ thread.subject }}
					</option>
				</select>
			</header>

			<p v-if="model.statusMessage" class="mail-list-view__status" :role="model.status === 'error' ? 'alert' : 'status'">
				{{ model.statusMessage }}
			</p>

			<div class="mail-list-view__items">
				<button
					v-for="message in visibleMessages"
					:key="message.id"
					type="button"
					class="mail-list-item"
					:class="{ selected: message.selected, unread: message.unread }"
					:aria-pressed="message.selected"
					@click="emit('selectMessage', message.id)"
				>
					<div class="mail-list-item__heading">
						<span class="mail-list-item__source"><Icon icon="tabler:mail" size="0.9rem" /></span>
						<strong>{{ message.sender }}</strong>
						<time>{{ message.meta }}</time>
					</div>
					<h3>{{ message.subject }}</h3>
					<div class="mail-list-item__summary">
						<p>{{ message.snippet }}</p>
						<span v-if="message.hasAttachments" title="Has attachments">
							<Icon icon="tabler:paperclip" size="0.9rem" />
						</span>
						<span class="mail-list-item__read-dot" :title="message.unread ? 'Unread' : 'Read'" />
					</div>
				</button>
				<p v-if="model.status !== 'loading' && visibleMessages.length === 0" class="mail-list-view__empty">
					No messages match this view.
				</p>
			</div>

			<button
				v-if="model.hasMoreMessages"
				type="button"
				class="mail-list-view__more"
				@click="emit('loadMore')"
			>
				Load more
			</button>
		</section>
	</aside>
</template>
