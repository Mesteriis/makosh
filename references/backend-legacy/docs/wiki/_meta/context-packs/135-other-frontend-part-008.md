# Задача для DeepSeek: обновить русскую Obsidian wiki

## Safety instructions / Инструкции безопасности

- Do not print, infer, summarize, or request secrets. / Не печатай, не выводи, не пересказывай и не запрашивай секреты.
- Treat `.env`, credential, token, key, certificate, and private paths as redacted even if referenced. / Считай `.env`, учетные данные, токены, ключи, сертификаты и приватные пути редактированными.
- Keep code identifiers, file paths, commands, package names, API names, and ADR titles exactly as written. / Сохраняй идентификаторы кода, пути, команды, имена пакетов, API и названия ADR без изменений.
- Write wiki prose in Russian and keep Markdown Obsidian-compatible. / Пиши текст wiki на русском и сохраняй совместимость с Obsidian Markdown.
- Do not invent source facts. If the context is insufficient, state that explicitly. / Не выдумывай факты об исходниках. Если контекста недостаточно, напиши это явно.
- Every behavioral statement in proposed wiki pages must be directly supported by the embedded source text. / Каждое утверждение о поведении в предлагаемых wiki-страницах должно напрямую подтверждаться встроенным текстом исходников.
- Do not infer semantics for profiles, flags, annotations, environment variables, or framework conventions unless this context pack explicitly defines them. / Не выводи семантику профилей, флагов, аннотаций, переменных окружения или framework-конвенций, если этот context pack явно её не определяет.
- Do not add external background knowledge about tools, frameworks, or CLIs. / Не добавляй внешние справочные знания об инструментах, framework или CLI.
- When only a command or config value is visible, document only the literal command or value. For deeper meaning, write only that it is not confirmed by this context. / Когда видна только команда или значение конфигурации, документируй только буквальную команду или значение. Для более глубокого смысла пиши только, что он не подтвержден этим контекстом.
- Do not name likely related files unless they are embedded in this context pack. / Не называй вероятные связанные файлы, если они не встроены в этот context pack.
- Use only the embedded Source Files section below. Do not call tools, read files, inspect the filesystem, or access MCP/web resources. / Используй только встроенный ниже раздел Source Files. Не вызывай tools, не читай файлы, не инспектируй файловую систему и не обращайся к MCP/web ресурсам.
- If a referenced path or wiki page is not embedded in this context pack, report insufficient context instead of trying to open it. / Если упомянутый путь или wiki-страница не встроены в этот context pack, укажи недостаток контекста вместо попытки открыть файл.

## Chunk details / Детали чанка

- Chunk ID / ID чанка: `135-other-frontend-part-008`
- Group / Группа: `frontend`
- Role / Роль: `other`
- Status / Статус: `pending`
- Repository / Репозиторий: `/Users/avm/projects/Personal/makosh`
- Wiki path / Путь wiki: `/Users/avm/projects/Personal/makosh/docs/wiki`
- Metadata path / Путь metadata: `/Users/avm/projects/Personal/makosh/docs/wiki/_meta`
- Plan generated at / План создан: `2026-06-28T19:48:55Z`
- Per-file source limit / Лимит источника на файл: `12000` characters

## Target pages / Целевые страницы

- `components/frontend.md`

## Required Output / Требуемый результат

Return one Markdown response with these sections and no extra wrapper text. / Верни один Markdown-ответ с этими разделами и без дополнительной обертки.

### Summary / Резюме

Briefly describe what should change in the Russian wiki and why. / Кратко опиши, что нужно изменить в русской wiki и почему.

### Proposed pages / Предлагаемые страницы

For each target page, provide the wiki-relative path and full proposed Obsidian-compatible Markdown content. / Для каждой целевой страницы укажи путь относительно wiki и полный предложенный Markdown, совместимый с Obsidian.

### Source coverage / Покрытие источников

List each source file and the facts from it that the proposed pages cover. / Перечисли каждый исходный файл и факты из него, покрытые предложенными страницами.

### Drift candidates / Кандидаты на drift

List possible code/docs/ADR drift found in this chunk, or state that none is visible from the provided context. / Перечисли возможные расхождения кода, документации и ADR в этом чанке либо укажи, что из данного контекста они не видны.

## Source Files / Исходные файлы

### `frontend/src/domains/communications/providers/whatsapp/views/WhatsAppCommunicationsChatPane.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/providers/whatsapp/views/WhatsAppCommunicationsChatPane.vue`
- Size bytes / Размер в байтах: `12358`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```text
<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from '../../../../../platform/i18n'
import Icon from '../../../../../shared/ui/Icon.vue'
import type {
	WhatsappWebMediaItem,
} from '../../../../../shared/communications/types/whatsapp'
import { TELEGRAM_REACTION_PALETTE } from '../../../../../shared/communications/types/telegram'
import type { CommunicationProviderConversation } from '../../../types/providerChannels'
import {
	isPreviewableMediaItem,
	isStatusMessage,
	mediaLabel,
	mediaMetaLabel,
	mediaTime,
	messageContactCardSummary as buildMessageContactCardSummary,
	messageLinkPreview,
	messageLocationSummary as buildMessageLocationSummary,
	messageMentionNames,
	messageMetaFlags as buildMessageMetaFlags,
	messagePollSummary as buildMessagePollSummary,
	messageStickerSummary as buildMessageStickerSummary,
	messageSystemSummary as buildMessageSystemSummary,
	messageTime,
	reactionSummary,
	statusAuthorDetail,
	statusAuthorHeadline,
	statusDeletedSummary as buildStatusDeletedSummary,
	statusMediaCountLabel as buildStatusMediaCountLabel,
	statusMessageMediaItems as collectStatusMessageMediaItems,
	statusViewSummary as buildStatusViewSummary,
	type WhatsAppPanelMessage,
} from './WhatsAppCommunicationsPanel.helpers'

const props = defineProps<{
	selectedConversation: CommunicationProviderConversation | null
	browserMode: 'timeline' | 'media'
	selectedMessages: WhatsAppPanelMessage[]
	mediaItems: WhatsappWebMediaItem[]
	draftText: string
	emptyStateMessage: string
	isBusy: boolean
	isConversationUnread: boolean
	isConversationMuted: boolean
	isConversationArchived: boolean
	isConversationPinned: boolean
}>()

const emit = defineEmits<{
	(event: 'update:draftText', value: string): void
	(event: 'toggle-unread'): void
	(event: 'toggle-mute'): void
	(event: 'toggle-archive'): void
	(event: 'toggle-pin'): void
	(event: 'send-message'): void
	(event: 'set-message-ref', messageId: string, element: unknown): void
	(event: 'reply', message: WhatsAppPanelMessage): void
	(event: 'forward', message: WhatsAppPanelMessage): void
	(event: 'edit', message: WhatsAppPanelMessage): void
	(event: 'delete', message: WhatsAppPanelMessage): void
	(event: 'select-media', item: WhatsappWebMediaItem): void
	(event: 'jump-to-message', messageId: string): void
	(event: 'add-reaction', message: WhatsAppPanelMessage, reactionEmoji: string): void
	(event: 'remove-reaction', message: WhatsAppPanelMessage, reactionEmoji: string): void
}>()

const { t } = useI18n()

const draftTextModel = computed({
	get: () => props.draftText,
	set: (value: string) => emit('update:draftText', value),
})

function statusMessageMediaItems(message: WhatsAppPanelMessage): WhatsappWebMediaItem[] {
	return collectStatusMessageMediaItems(message, props.mediaItems)
}

function statusViewSummary(message: WhatsAppPanelMessage): string | null {
	return buildStatusViewSummary(message, t)
}

function statusDeletedSummary(message: WhatsAppPanelMessage): string | null {
	return buildStatusDeletedSummary(message, t)
}

function statusMediaCountLabel(message: WhatsAppPanelMessage): string | null {
	return buildStatusMediaCountLabel(message, props.mediaItems, t)
}

function messageMetaFlags(message: WhatsAppPanelMessage): string[] {
	return buildMessageMetaFlags(message, t)
}

function messagePollSummary(message: WhatsAppPanelMessage): string | null {
	return buildMessagePollSummary(message, t)
}

function messageLocationSummary(message: WhatsAppPanelMessage): string | null {
	return buildMessageLocationSummary(message, t)
}

function messageContactCardSummary(message: WhatsAppPanelMessage): string | null {
	return buildMessageContactCardSummary(message, t)
}

function messageStickerSummary(message: WhatsAppPanelMessage): string | null {
	return buildMessageStickerSummary(message, t)
}

function messageSystemSummary(message: WhatsAppPanelMessage): string | null {
	return buildMessageSystemSummary(message, t)
}
</script>

<template>
	<section class="panel chat-pane">
		<header class="provider-thread-header">
			<div>
				<h2>{{ selectedConversation?.title ?? t('No conversation selected') }}</h2>
				<p>{{ selectedConversation?.account_id ?? '' }}</p>
			</div>
			<div class="thread-actions">
				<button type="button" :disabled="isBusy || !selectedConversation?.conversation_id" @click="emit('toggle-unread')">
					<Icon :icon="isConversationUnread ? 'tabler:mail-opened' : 'tabler:mail'" width="14" height="14" />
					{{ isConversationUnread ? t('Mark read') : t('Mark unread') }}
				</button>
				<button type="button" :disabled="isBusy || !selectedConversation?.conversation_id" @click="emit('toggle-mute')">
					<Icon :icon="isConversationMuted ? 'tabler:volume' : 'tabler:volume-off'" width="14" height="14" />
					{{ isConversationMuted ? t('Unmute') : t('Mute') }}
				</button>
				<button type="button" :disabled="isBusy || !selectedConversation?.conversation_id" @click="emit('toggle-archive')">
					<Icon :icon="isConversationArchived ? 'tabler:archive-off' : 'tabler:archive'" width="14" height="14" />
					{{ isConversationArchived ? t('Unarchive') : t('Archive') }}
				</button>
				<button type="button" :disabled="isBusy || !selectedConversation?.conversation_id" @click="emit('toggle-pin')">
					<Icon :icon="isConversationPinned ? 'tabler:pinned-off' : 'tabler:pin'" width="14" height="14" />
					{{ isConversationPinned ? t('Unpin') : t('Pin') }}
				</button>
			</div>
		</header>
		<div class="message-scroll">
			<template v-if="browserMode === 'timeline'">
				<article
					v-for="message in selectedMessages"
					:key="message.message_id"
					class="message-bubble"
					:ref="(element) => emit('set-message-ref', message.message_id, element)"
				>
					<header>
						<strong>{{ message.sender_display_name ?? message.sender }}</strong>
						<time>{{ messageTime(message) }}</time>
					</header>
					<div v-if="messageMetaFlags(message).length" class="message-meta-flags">
						<span v-for="flag in messageMetaFlags(message)" :key="`${message.message_id}:${flag}`" class="message-meta-chip">
							{{ flag }}
						</span>
					</div>
					<p>{{ message.text }}</p>
					<ul v-if="messageMentionNames(message).length" class="message-meta-list">
						<li>
							<strong>{{ t('Mentions') }}</strong>
							<span>{{ messageMentionNames(message).join(', ') }}</span>
						</li>
					</ul>
					<ul v-if="isStatusMessage(message)" class="message-meta-list">
						<li v-if="statusAuthorHeadline(message)"><strong>{{ t('Status author') }}</strong><span>{{ statusAuthorHeadline(message) }}</span></li>
						<li v-if="statusAuthorDetail(message)"><strong>{{ t('Identity') }}</strong><span>{{ statusAuthorDetail(message) }}</span></li>
						<li v-if="statusViewSummary(message)"><strong>{{ t('Views') }}</strong><span>{{ statusViewSummary(message) }}</span></li>
						<li v-if="statusDeletedSummary(message)"><strong>{{ t('Lifecycle') }}</strong><span>{{ statusDeletedSummary(message) }}</span></li>
						<li v-if="statusMediaCountLabel(message)"><strong>{{ t('Status media') }}</strong><span>{{ statusMediaCountLabel(message) }}</span></li>
					</ul>
					<div v-if="messageLinkPreview(message)" class="message-meta-card">
						<strong>{{ messageLinkPreview(message)?.title ?? t('Link preview') }}</strong>
						<span>{{ messageLinkPreview(message)?.site ?? messageLinkPreview(message)?.url }}</span>
					</div>
					<ul
						v-if="messagePollSummary(message) || messageLocationSummary(message) || messageContactCardSummary(message) || messageStickerSummary(message) || messageSystemSummary(message)"
						class="message-meta-list"
					>
						<li v-if="messagePollSummary(message)"><strong>{{ t('Poll') }}</strong><span>{{ messagePollSummary(message) }}</span></li>
						<li v-if="messageLocationSummary(message)"><strong>{{ t('Location') }}</strong><span>{{ messageLocationSummary(message) }}</span></li>
						<li v-if="messageContactCardSummary(message)"><strong>{{ t('Contact') }}</strong><span>{{ messageContactCardSummary(message) }}</span></li>
						<li v-if="messageStickerSummary(message)"><strong>{{ t('Sticker') }}</strong><span>{{ messageStickerSummary(message) }}</span></li>
						<li v-if="messageSystemSummary(message)"><strong>{{ t('Event') }}</strong><span>{{ messageSystemSummary(message) }}</span></li>
					</ul>
					<div v-if="isStatusMessage(message) && statusMessageMediaItems(message).length" class="status-media-grid">
						<article
							v-for="item in statusMessageMediaItems(message)"
							:key="`${message.message_id}:${item.attachment_id ?? item.provider_attachment_id ?? item.file_name}`"
							class="status-media-card"
						>
							<strong>{{ mediaLabel(item) }}</strong>
							<span>{{ mediaMetaLabel(item) }}</span>
							<div class="status-media-actions">
								<button v-if="isPreviewableMediaItem(item)" type="button" :disabled="isBusy" @click="emit('select-media', item)">
									<Icon icon="tabler:photo-search" width="14" height="14" />{{ t('Preview media') }}
								</button>
								<button type="button" :disabled="isBusy" @click="emit('jump-to-message', item.message_id)">
									<Icon icon="tabler:arrow-up-right" width="14" height="14" />{{ t('Open source message') }}
								</button>
							</div>
						</article>
					</div>
					<div v-if="reactionSummary(message).length" class="message-reactions">
						<button
							v-for="reaction in reactionSummary(message)"
							:key="`${message.message_id}:${reaction.reaction_emoji}`"
							type="button"
							class="reaction-chip"
							:disabled="isBusy"
							@click="emit('remove-reaction', message, reaction.reaction_emoji)"
						>
							<span>{{ reaction.reaction_emoji }}</span>
							<span>{{ reaction.count }}</span>
						</button>
					</div>
					<footer>
						<button type="button" :disabled="isBusy || !draftTextModel.trim()" @click="emit('reply', message)">
							<Icon icon="tabler:message-reply" width="14" height="14" />{{ t('Reply') }}
						</button>
						<button type="button" :disabled="isBusy" @click="emit('forward', message)">
							<Icon icon="tabler:arrow-forward-up" width="14" height="14" />{{ t('Forward') }}
						</button>
						<button type="button" :disabled="isBusy" @click="emit('edit', message)">
							<Icon icon="tabler:edit" width="14" height="14" />{{ t('Edit') }}
						</button>
						<button type="button" :disabled="isBusy" @click="emit('delete', message)">
							<Icon icon="tabler:trash" width="14" height="14" />{{ t('Delete') }}
						</button>
					</footer>
					<div class="reaction-palette">
						<button
							v-for="reactionEmoji in TELEGRAM_REACTION_PALETTE.slice(0, 8)"
							:key="`${message.message_id}:palette:${reactionEmoji}`"
							type="button"
							class="reaction-chip add"
							:disabled="isBusy"
							@click="emit('add-reaction', message, reactionEmoji)"
						>
							{{ reactionEmoji }}
						</button>
					</div>
				</article>
			</template>
			<div v-else class="media-gallery">
				<article v-for="item in mediaItems" :key="`${item.message_id}:${item.provider_attachment_id ?? item.file_name}`" class="media-card">
					<header>
						<strong>{{ mediaLabel(item) }}</strong>
						<time>{{ mediaTime(item) }}</time>
					</header>
					<p>{{ mediaMetaLabel(item) }}</p>
					<footer>
						<button v-if="isPreviewableMediaItem(item)" type="button" :disabled="isBusy" @click="emit('select-media', item)">
							<Icon icon="tabler:photo-search" width="14" height="14" />{{ t('Preview media') }}
						</button>
						<button type="button" :disabled="isBusy" @click="emit('jump-to-message', item.message_id)">
							<Icon icon="tabler:arrow-up-right" width="14" height="14" />{{ t('Open source message') }}
						</button>
					</footer>
				</article>
			</div>
			<div v-if="(browserMode === 'timeline' && !selectedMessages.length) || (browserMode === 'media' && !mediaItems.length)" class="empty-panel">
				{{ emptyStateMessage }}
			</div>
		</div>
		<form class="provider-inline-form"
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `frontend/src/domains/communications/providers/whatsapp/views/WhatsAppCommunicationsDetailPane.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/providers/whatsapp/views/WhatsAppCommunicationsDetailPane.vue`
- Size bytes / Размер в байтах: `10409`
- Included characters / Включено символов: `10408`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from '../../../../../platform/i18n'
import Icon from '../../../../../shared/ui/Icon.vue'
import type {
	WhatsappWebMediaItem,
} from '../../../../../shared/communications/types/whatsapp'
import type { TelegramChatMember } from '../../../../../shared/communications/types/telegramMembers'
import type { AttachmentPreviewResponse } from '../../../types/attachments'
import type { CommunicationProviderConversation } from '../../../types/providerChannels'
import {
	isPreviewableMediaItem,
	mediaLabel,
	mediaMetaLabel,
	memberLabel,
	messagePreview as buildMessagePreview,
	type WhatsAppPanelMessage,
} from './WhatsAppCommunicationsPanel.helpers'

const props = defineProps<{
	selectedConversation: CommunicationProviderConversation | null
	isStatusFeedConversation: boolean
	isConversationUnread: boolean
	isConversationArchived: boolean
	isConversationMuted: boolean
	isConversationPinned: boolean
	participantCount: number
	editingMessage: WhatsAppPanelMessage | null
	editDraftText: string
	forwardingMessage: WhatsAppPanelMessage | null
	forwardTargetFilter: string
	forwardTargetConversationId: string
	forwardTargetConversations: CommunicationProviderConversation[]
	members: TelegramChatMember[]
	pinnedMessages: WhatsAppPanelMessage[]
	mediaItems: WhatsappWebMediaItem[]
	selectedMediaItem: WhatsappWebMediaItem | null
	mediaPreview: AttachmentPreviewResponse | null
	mediaPreviewError: string
	isMediaPreviewFetching: boolean
	isBusy: boolean
}>()

const emit = defineEmits<{
	(event: 'update:editDraftText', value: string): void
	(event: 'update:forwardTargetFilter', value: string): void
	(event: 'update:forwardTargetConversationId', value: string): void
	(event: 'confirm-edit'): void
	(event: 'cancel-edit'): void
	(event: 'confirm-forward'): void
	(event: 'cancel-forward'): void
	(event: 'select-media', item: WhatsappWebMediaItem): void
	(event: 'clear-media-preview'): void
	(event: 'jump-to-message', messageId: string): void
}>()

const { t } = useI18n()

const editDraftTextModel = computed({
	get: () => props.editDraftText,
	set: (value: string) => emit('update:editDraftText', value),
})
const forwardTargetFilterModel = computed({
	get: () => props.forwardTargetFilter,
	set: (value: string) => emit('update:forwardTargetFilter', value),
})
const forwardTargetConversationIdModel = computed({
	get: () => props.forwardTargetConversationId,
	set: (value: string) => emit('update:forwardTargetConversationId', value),
})

function messagePreview(message: { text?: string; body_text_preview?: string | null }): string {
	return buildMessagePreview(message, t)
}
</script>

<template>
	<aside class="panel detail-pane">
		<header class="provider-panel-header">
			<h2>{{ t('Details') }}</h2>
		</header>
		<div class="detail-scroll">
			<section class="detail-section">
				<dl class="detail-list">
					<div><dt>{{ t('Kind') }}</dt><dd>{{ isStatusFeedConversation ? t('status_feed') : (selectedConversation?.chat_kind ?? t('unknown')) }}</dd></div>
					<div><dt>{{ t('Unread') }}</dt><dd>{{ isConversationUnread ? t('Yes') : t('No') }}</dd></div>
					<div><dt>{{ t('Participants') }}</dt><dd>{{ participantCount }}</dd></div>
					<div><dt>{{ t('Archived') }}</dt><dd>{{ isConversationArchived ? t('Yes') : t('No') }}</dd></div>
					<div><dt>{{ t('Muted') }}</dt><dd>{{ isConversationMuted ? t('Yes') : t('No') }}</dd></div>
					<div><dt>{{ t('Pinned chat') }}</dt><dd>{{ isConversationPinned ? t('Yes') : t('No') }}</dd></div>
				</dl>
			</section>

			<section class="detail-section">
				<h3>{{ t('Edit draft') }}</h3>
				<div v-if="editingMessage" class="forward-panel">
					<p class="forward-panel-copy">
						{{ editingMessage.sender_display_name ?? editingMessage.sender }}:
						{{ messagePreview(editingMessage) }}
					</p>
					<label class="runtime-field">
						<span>{{ t('Edited text') }}</span>
						<textarea v-model="editDraftTextModel" rows="4" maxlength="4000" :placeholder="t('Update message text')" />
					</label>
					<div class="thread-actions">
						<button type="button" :disabled="isBusy || !editDraftTextModel.trim()" @click="emit('confirm-edit')">
							<Icon icon="tabler:check" width="14" height="14" />{{ t('Save edit') }}
						</button>
						<button type="button" :disabled="isBusy" @click="emit('cancel-edit')">
							<Icon icon="tabler:x" width="14" height="14" />{{ t('Cancel') }}
						</button>
					</div>
				</div>
				<div v-else class="empty-panel compact">{{ t('Choose Edit on a message to change it here.') }}</div>
			</section>

			<section class="detail-section">
				<h3>{{ t('Forward target') }}</h3>
				<div v-if="forwardingMessage" class="forward-panel">
					<p class="forward-panel-copy">
						{{ forwardingMessage.sender_display_name ?? forwardingMessage.sender }}:
						{{ messagePreview(forwardingMessage) }}
					</p>
					<label class="provider-search compact">
						<Icon icon="tabler:search" width="14" height="14" />
						<input v-model="forwardTargetFilterModel" type="search" :placeholder="t('Filter target conversations')" />
					</label>
					<div class="forward-target-list">
						<button
							v-for="conversation in forwardTargetConversations.slice(0, 10)"
							:key="conversation.conversation_id ?? conversation.provider_chat_id"
							type="button"
							class="provider-row compact"
							:class="{ active: forwardTargetConversationIdModel === conversation.provider_chat_id }"
							@click="forwardTargetConversationIdModel = conversation.provider_chat_id"
						>
							<strong>{{ conversation.title }}</strong>
							<span>{{ conversation.provider_chat_id }}</span>
						</button>
					</div>
					<div class="thread-actions">
						<button type="button" :disabled="isBusy || !forwardTargetConversationIdModel.trim()" @click="emit('confirm-forward')">
							<Icon icon="tabler:send-2" width="14" height="14" />{{ t('Forward here') }}
						</button>
						<button type="button" :disabled="isBusy" @click="emit('cancel-forward')">
							<Icon icon="tabler:x" width="14" height="14" />{{ t('Cancel') }}
						</button>
					</div>
				</div>
				<div v-else class="empty-panel compact">{{ t('Choose Forward on a message to pick a target conversation here.') }}</div>
			</section>

			<section class="detail-section">
				<h3>{{ t('Members') }}</h3>
				<ul v-if="members.length" class="detail-stack">
					<li v-for="member in members.slice(0, 8)" :key="member.provider_member_id">
						<strong>{{ memberLabel(member) }}</strong>
						<span>{{ member.role ?? member.status ?? t('member') }}</span>
					</li>
				</ul>
				<div v-else class="empty-panel compact">{{ t('No projected members yet.') }}</div>
			</section>

			<section class="detail-section">
				<h3>{{ t('Pinned messages') }}</h3>
				<ul v-if="pinnedMessages.length" class="detail-stack">
					<li v-for="message in pinnedMessages.slice(0, 5)" :key="message.message_id">
						<strong>{{ message.sender_display_name ?? message.sender }}</strong>
						<span>{{ messagePreview(message) }}</span>
						<button type="button" :disabled="isBusy" @click="emit('jump-to-message', message.message_id)">
							<Icon icon="tabler:arrow-up-right" width="14" height="14" />{{ t('Jump to message') }}
						</button>
					</li>
				</ul>
				<div v-else class="empty-panel compact">{{ t('No pinned messages.') }}</div>
			</section>

			<section class="detail-section">
				<h3>{{ t('Media') }}</h3>
				<ul v-if="mediaItems.length" class="detail-stack">
					<li v-for="item in mediaItems.slice(0, 6)" :key="`${item.message_id}:${item.provider_attachment_id ?? item.file_name}`">
						<strong>{{ mediaLabel(item) }}</strong>
						<span>{{ item.kind }}{{ item.mime_type ? ` · ${item.mime_type}` : '' }}</span>
						<button v-if="isPreviewableMediaItem(item)" type="button" :disabled="isBusy" @click="emit('select-media', item)">
							<Icon icon="tabler:photo-search" width="14" height="14" />{{ t('Preview media') }}
						</button>
						<button type="button" :disabled="isBusy" @click="emit('jump-to-message', item.message_id)">
							<Icon icon="tabler:arrow-up-right" width="14" height="14" />{{ t('Open source message') }}
						</button>
					</li>
				</ul>
				<div v-else class="empty-panel compact">{{ t('No projected media yet.') }}</div>
			</section>

			<section class="detail-section">
				<h3>{{ t('Media preview') }}</h3>
				<div v-if="selectedMediaItem" class="media-preview-panel">
					<div class="media-preview-header">
						<div>
							<strong>{{ mediaLabel(selectedMediaItem) }}</strong>
							<span>{{ mediaMetaLabel(selectedMediaItem) }}</span>
						</div>
						<button type="button" :disabled="isBusy" @click="emit('clear-media-preview')">
							<Icon icon="tabler:x" width="14" height="14" />{{ t('Close preview') }}
						</button>
					</div>
					<p v-if="isMediaPreviewFetching" class="media-preview-copy">{{ t('Loading safe media preview') }}</p>
					<p v-else-if="mediaPreviewError" class="media-preview-copy error">{{ mediaPreviewError }}</p>
					<div v-else-if="mediaPreview" class="media-preview-body">
						<img
							v-if="mediaPreview.preview_kind === 'image' && mediaPreview.data_url"
							class="media-preview-image"
							:src="mediaPreview.data_url"
							:alt="mediaLabel(selectedMediaItem)"
						/>
						<audio v-else-if="mediaPreview.preview_kind === 'audio' && mediaPreview.data_url" class="media-preview-media" controls preload="metadata" :src="mediaPreview.data_url" />
						<video v-else-if="mediaPreview.preview_kind === 'video' && mediaPreview.data_url" class="media-preview-media" controls preload="metadata" :src="mediaPreview.data_url" />
						<iframe v-else-if="mediaPreview.preview_kind === 'pdf' && mediaPreview.data_url" class="media-preview-document" :src="mediaPreview.data_url" :title="mediaLabel(selectedMediaItem)" />
						<pre v-else class="media-preview-text">{{ mediaPreview.text }}</pre>
						<p class="media-preview-copy">
							{{ mediaPreview.truncated ? t('Preview is truncated to the safe local limit.') : t('Preview loaded from the local attachment blob.') }}
						</p>
					</div>
					<p v-else class="media-preview-copy">{{ t('Select previewable media to open it here.') }}</p>
				</div>
				<div v-else class="empty-panel compact">{{ t('Select previewable media to open it here.') }}</div>
			</section>
		</div>
	</aside>
</template>
```

### `frontend/src/domains/communications/providers/whatsapp/views/WhatsAppCommunicationsPanel.css`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/providers/whatsapp/views/WhatsAppCommunicationsPanel.css`
- Size bytes / Размер в байтах: `7076`
- Included characters / Включено символов: `7076`
- Truncated / Обрезано: `no`

```text
.whatsapp-communications-panel {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
}
.view-header,
.view-title-with-icon,
.provider-panel-header,
.provider-thread-header,
.provider-search,
.header-actions,
.message-bubble header,
.message-bubble footer,
.provider-inline-form {
  display: flex;
  align-items: center;
  gap: 0.75rem;
}
.view-header,
.provider-panel-header,
.provider-thread-header {
  justify-content: space-between;
  padding: 0.75rem 1rem;
  border-bottom: 1px solid var(--hh-border, #d9e2ec);
}
.header-actions {
  margin-left: auto;
}
.thread-actions {
  display: flex;
  flex-wrap: wrap;
  justify-content: flex-end;
  gap: 0.5rem;
}
.thread-actions button.active {
  background: var(--hh-bg-muted, #f5f8fb);
}
.provider-search {
  border: 1px solid var(--hh-border, #d9e2ec);
  border-radius: 8px;
  padding: 0.4rem 0.6rem;
  background: var(--hh-bg-primary, #fff);
}
.media-filter {
  min-width: 9rem;
}
.provider-search input,
.provider-inline-form input {
  border: 0;
  outline: 0;
  min-width: 220px;
  background: transparent;
  color: inherit;
}
.provider-inline-form {
  border-top: 1px solid var(--hh-border, #d9e2ec);
  padding: 0.75rem 1rem;
  background: var(--hh-bg-primary, #fff);
}
.provider-list-scroll,
.message-scroll,
.detail-scroll {
  flex: 1;
  overflow: auto;
  min-height: 0;
}
.provider-row {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
  width: 100%;
  padding: 0.7rem 0.85rem;
  border: 0;
  border-bottom: 1px solid var(--hh-border, #eef2f6);
  background: transparent;
  color: inherit;
  text-align: left;
  cursor: pointer;
}
.provider-row.active,
.provider-row:hover {
  background: var(--hh-bg-muted, #f5f8fb);
}
.provider-row.compact {
  padding: 0.55rem 0.7rem;
}
.provider-row span,
.message-bubble time {
  color: var(--hh-text-muted, #667085);
  font-size: 0.78rem;
}
.message-bubble {
  margin: 0.75rem;
  padding: 0.75rem;
  border: 1px solid var(--hh-border, #d9e2ec);
  border-radius: 8px;
  background: var(--hh-bg-primary, #fff);
  transition: box-shadow 0.2s ease, border-color 0.2s ease, background 0.2s ease;
}
.message-bubble--flash {
  border-color: var(--hh-accent, #16a34a);
  box-shadow: 0 0 0 1px var(--hh-accent, #16a34a);
  background: color-mix(in srgb, var(--hh-bg-primary, #fff) 92%, var(--hh-accent, #16a34a));
}
.message-bubble header {
  justify-content: space-between;
  margin-bottom: 0.35rem;
}
.message-meta-flags {
  display: flex;
  flex-wrap: wrap;
  gap: 0.4rem;
  margin-bottom: 0.5rem;
}
.message-meta-chip {
  display: inline-flex;
  align-items: center;
  min-height: 1.6rem;
  padding: 0 0.55rem;
  border: 1px solid var(--hh-border, #d9e2ec);
  border-radius: 999px;
  background: var(--hh-bg-muted, #f5f8fb);
  color: var(--hh-text-muted, #667085);
  font-size: 0.75rem;
}
.message-meta-card {
  display: grid;
  gap: 0.15rem;
  margin-top: 0.5rem;
  padding: 0.6rem 0.7rem;
  border: 1px solid var(--hh-border, #d9e2ec);
  border-radius: 8px;
  background: var(--hh-bg-muted, #f5f8fb);
}
.message-meta-card span {
  color: var(--hh-text-muted, #667085);
  font-size: 0.8rem;
}
.message-meta-list {
  display: grid;
  gap: 0.35rem;
  margin: 0.55rem 0 0;
  padding: 0;
  list-style: none;
}
.message-meta-list li {
  display: grid;
  gap: 0.12rem;
}
.message-meta-list span {
  color: var(--hh-text-muted, #667085);
  font-size: 0.8rem;
}
.message-bubble footer {
  flex-wrap: wrap;
  margin-top: 0.5rem;
}
.message-reactions,
.reaction-palette {
  display: flex;
  flex-wrap: wrap;
  gap: 0.45rem;
  margin-top: 0.55rem;
}
.media-gallery {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(220px, 1fr));
  gap: 0.75rem;
  padding: 0.75rem;
}
.media-card {
  display: grid;
  gap: 0.5rem;
  padding: 0.75rem;
  border: 1px solid var(--hh-border, #d9e2ec);
  border-radius: 8px;
  background: var(--hh-bg-primary, #fff);
}
.media-card header,
.media-card footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.5rem;
}
.media-card p {
  margin: 0;
  color: var(--hh-text-muted, #667085);
  font-size: 0.85rem;
}
.status-media-grid {
  display: grid;
  gap: 0.55rem;
  margin-top: 0.65rem;
}
.status-media-card {
  display: grid;
  gap: 0.35rem;
  padding: 0.65rem 0.7rem;
  border: 1px solid var(--hh-border, #d9e2ec);
  border-radius: 8px;
  background: var(--hh-bg-muted, #f5f8fb);
}
.status-media-card span {
  color: var(--hh-text-muted, #667085);
  font-size: 0.8rem;
}
.status-media-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 0.45rem;
}
.media-preview-panel,
.media-preview-body {
  display: grid;
  gap: 0.65rem;
}
.media-preview-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 0.75rem;
}
.media-preview-header span,
.media-preview-copy {
  color: var(--hh-text-muted, #667085);
  font-size: 0.8rem;
}
.media-preview-copy.error {
  color: var(--hh-danger, #b42318);
}
.media-preview-image {
  width: 100%;
  max-height: 18rem;
  object-fit: contain;
  border: 1px solid var(--hh-border, #d9e2ec);
  border-radius: 8px;
  background: var(--hh-bg-muted, #f5f8fb);
}
.media-preview-media {
  width: 100%;
  max-height: 18rem;
  border: 1px solid var(--hh-border, #d9e2ec);
  border-radius: 8px;
  background: var(--hh-bg-muted, #f5f8fb);
}
.media-preview-document {
  width: 100%;
  min-height: 20rem;
  border: 1px solid var(--hh-border, #d9e2ec);
  border-radius: 8px;
  background: var(--hh-bg-muted, #f5f8fb);
}
.media-preview-text {
  margin: 0;
  max-height: 16rem;
  overflow: auto;
  padding: 0.75rem;
  border: 1px solid var(--hh-border, #d9e2ec);
  border-radius: 8px;
  background: var(--hh-bg-muted, #f5f8fb);
  font-size: 0.78rem;
  white-space: pre-wrap;
}
.reaction-chip {
  display: inline-flex;
  align-items: center;
  gap: 0.35rem;
  min-height: 2rem;
  padding: 0 0.7rem;
  border: 1px solid var(--hh-border, #d9e2ec);
  border-radius: 999px;
  background: var(--hh-bg-muted, #f5f8fb);
  color: inherit;
}
.reaction-chip.add {
  background: var(--hh-bg-primary, #fff);
}
.detail-pane {
  min-width: 280px;
  max-width: 360px;
}
.detail-section {
  padding: 0.85rem 1rem;
  border-bottom: 1px solid var(--hh-border, #eef2f6);
}
.detail-section h3 {
  margin: 0 0 0.65rem;
  font-size: 0.92rem;
}
.forward-panel {
  display: grid;
  gap: 0.75rem;
}
.forward-panel-copy {
  margin: 0;
  color: var(--hh-text-muted, #667085);
  font-size: 0.85rem;
}
.forward-target-list {
  display: grid;
  gap: 0.4rem;
  max-height: 14rem;
  overflow: auto;
}
.detail-list {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 0.75rem;
  margin: 0;
}
.detail-list dt,
.detail-stack span {
  color: var(--hh-text-muted, #667085);
  font-size: 0.78rem;
}
.detail-list dd {
  margin: 0.15rem 0 0;
  font-weight: 600;
}
.detail-stack {
  display: flex;
  flex-direction: column;
  gap: 0.6rem;
  margin: 0;
  padding: 0;
  list-style: none;
}
.detail-stack li {
  display: flex;
  flex-direction: column;
  gap: 0.15rem;
}
.empty-panel {
  padding: 0.75rem;
}
.empty-panel.compact {
  padding: 0;
}
```

### `frontend/src/domains/communications/providers/whatsapp/views/WhatsAppCommunicationsPanel.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/providers/whatsapp/views/WhatsAppCommunicationsPanel.vue`
- Size bytes / Размер в байтах: `25855`
- Included characters / Включено символов: `12000`
- Truncated / Обрезано: `yes`

```text
<script setup lang="ts">
import { computed, nextTick, ref, watch } from 'vue'
import { useI18n } from '../../../../../platform/i18n'
import Icon from '../../../../../shared/ui/Icon.vue'
import { useAttachmentPreviewQuery } from '../../../queries/useCommunicationsQuery'
import WhatsAppCommunicationsChatPane from './WhatsAppCommunicationsChatPane.vue'
import WhatsAppCommunicationsDetailPane from './WhatsAppCommunicationsDetailPane.vue'
import type {
  WhatsappWebMediaItem,
} from '../../../../../shared/communications/types/whatsapp'
import {
  useWhatsappBusinessConversationsQuery,
  useAddWhatsappReactionMutation,
  useArchiveWhatsappConversationMutation,
  useMarkWhatsappConversationReadMutation,
  useMarkWhatsappConversationUnreadMutation,
  useMuteWhatsappConversationMutation,
  useWhatsappConversationDetailQuery,
  useWhatsappConversationMembersQuery,
  useWhatsappBusinessMessagesQuery,
  useDeleteWhatsappMessageMutation,
  useEditWhatsappMessageMutation,
  useForwardWhatsappMessageMutation,
  useRemoveWhatsappReactionMutation,
  useWhatsappMediaSearchQuery,
  useWhatsappMessageSearchQuery,
  useWhatsappPinnedMessagesQuery,
  usePinWhatsappConversationMutation,
  useReplyWhatsappMessageMutation,
  useSendWhatsappMessageMutation,
  useUnarchiveWhatsappConversationMutation,
  useUnmuteWhatsappConversationMutation,
  useUnpinWhatsappConversationMutation,
} from '../../../queries/whatsappBusinessQueries'
import {
  firstPreviewableMediaAttachmentId,
  mediaAttachmentId,
  type WhatsAppPanelMessage,
} from './WhatsAppCommunicationsPanel.helpers'

const { t } = useI18n()
const selectedConversationId = ref('')
const draftText = ref('')
const searchText = ref('')
const browserMode = ref<'timeline' | 'media'>('timeline')
const mediaKindFilter = ref<'all' | 'image' | 'video' | 'audio' | 'document'>('all')
const selectedMediaAttachmentId = ref<string | null>(null)
const actionMessage = ref('')
const actionError = ref('')
const editingMessageId = ref<string | null>(null)
const editDraftText = ref('')
const forwardingMessageId = ref<string | null>(null)
const forwardTargetConversationId = ref('')
const forwardTargetFilter = ref('')
const messageElementMap = new Map<string, HTMLElement>()
const conversationsQuery = useWhatsappBusinessConversationsQuery(undefined, 200)
const conversations = computed(() => conversationsQuery.data.value ?? [])
const selectedConversationSummary = computed(
  () =>
    conversations.value.find(
      (conversation) => conversation.provider_chat_id === selectedConversationId.value
    ) ?? null
)
const conversationDetailQuery = useWhatsappConversationDetailQuery(
  () => selectedConversationSummary.value?.conversation_id ?? null
)
const selectedConversation = computed(
  () => conversationDetailQuery.data.value ?? selectedConversationSummary.value
)
const messagesQuery = useWhatsappBusinessMessagesQuery(
  () => selectedConversation.value?.account_id ?? null,
  () => selectedConversation.value?.provider_chat_id ?? null,
  200
)
const searchQuery = useWhatsappMessageSearchQuery({
  q: searchText,
  accountId: () => selectedConversation.value?.account_id ?? null,
  providerChatId: () => selectedConversation.value?.provider_chat_id ?? null,
  limit: 50,
})
const pinnedMessagesQuery = useWhatsappPinnedMessagesQuery({
  conversationId: () => selectedConversation.value?.conversation_id ?? null,
  limit: 20,
})
const membersQuery = useWhatsappConversationMembersQuery(
  () => selectedConversation.value?.conversation_id ?? null,
  50
)
const mediaQuery = useWhatsappMediaSearchQuery({
  q: () => searchText.value.trim() || undefined,
  accountId: () => selectedConversation.value?.account_id ?? null,
  providerChatId: () => selectedConversation.value?.provider_chat_id ?? null,
  kind: () => (mediaKindFilter.value === 'all' ? undefined : mediaKindFilter.value),
  limit: 20,
})
const sendMutation = useSendWhatsappMessageMutation()
const replyMutation = useReplyWhatsappMessageMutation()
const forwardMutation = useForwardWhatsappMessageMutation()
const editMutation = useEditWhatsappMessageMutation()
const deleteMutation = useDeleteWhatsappMessageMutation()
const pinConversationMutation = usePinWhatsappConversationMutation()
const unpinConversationMutation = useUnpinWhatsappConversationMutation()
const archiveConversationMutation = useArchiveWhatsappConversationMutation()
const unarchiveConversationMutation = useUnarchiveWhatsappConversationMutation()
const muteConversationMutation = useMuteWhatsappConversationMutation()
const unmuteConversationMutation = useUnmuteWhatsappConversationMutation()
const markConversationReadMutation = useMarkWhatsappConversationReadMutation()
const markConversationUnreadMutation = useMarkWhatsappConversationUnreadMutation()
const addReactionMutation = useAddWhatsappReactionMutation()
const removeReactionMutation = useRemoveWhatsappReactionMutation()
const messages = computed(() => messagesQuery.data.value ?? [])
const selectedMessages = computed(() =>
  searchText.value.trim() ? searchQuery.data.value?.items ?? [] : messages.value
)
const pinnedMessages = computed(() => pinnedMessagesQuery.data.value?.items ?? [])
const members = computed(() => membersQuery.data.value ?? [])
const mediaItems = computed(() => mediaQuery.data.value?.items ?? [])
const selectedMediaItem = computed(
  () => mediaItems.value.find((item) => item.attachment_id === selectedMediaAttachmentId.value) ?? null
)
const mediaPreviewQuery = useAttachmentPreviewQuery(
  () => selectedMediaAttachmentId.value,
  () => Boolean(selectedMediaAttachmentId.value)
)
const mediaPreview = computed(() => mediaPreviewQuery.data.value ?? null)
const mediaPreviewError = computed(() => {
  const error = mediaPreviewQuery.error.value
  if (!error) return ''
  return error instanceof Error ? error.message : t('Media preview failed')
})
const isStatusFeedConversation = computed(() =>
  selectedConversation.value?.provider_chat_id === 'status-feed' ||
  selectedConversation.value?.chat_kind === 'status_feed' ||
  Boolean(selectedConversation.value?.metadata?.is_status_feed)
)
const isSearchActive = computed(() => searchText.value.trim().length >= 2)
const emptyStateMessage = computed(() => {
  if (browserMode.value === 'media') {
    return isSearchActive.value ? t('No WhatsApp media search matches.') : t('No projected media yet.')
  }
  return searchText.value.trim() ? t('No WhatsApp search matches.') : t('Select a WhatsApp conversation.')
})
const editingMessage = computed(
  () => messages.value.find((message) => message.message_id === editingMessageId.value) ?? null
)
const forwardingMessage = computed(
  () => messages.value.find((message) => message.message_id === forwardingMessageId.value) ?? null
)
const forwardTargetConversations = computed(() => {
  const query = forwardTargetFilter.value.trim().toLowerCase()
  return conversations.value.filter((conversation) => {
    if (!query) return true
    return (
      conversation.title.toLowerCase().includes(query) ||
      conversation.provider_chat_id.toLowerCase().includes(query) ||
      conversation.account_id.toLowerCase().includes(query)
    )
  })
})
const isBusy = computed(() =>
  sendMutation.isPending.value ||
  replyMutation.isPending.value ||
  forwardMutation.isPending.value ||
  editMutation.isPending.value ||
  deleteMutation.isPending.value ||
  pinConversationMutation.isPending.value ||
  unpinConversationMutation.isPending.value ||
  archiveConversationMutation.isPending.value ||
  unarchiveConversationMutation.isPending.value ||
  muteConversationMutation.isPending.value ||
  unmuteConversationMutation.isPending.value ||
  markConversationReadMutation.isPending.value ||
  markConversationUnreadMutation.isPending.value ||
  addReactionMutation.isPending.value ||
  removeReactionMutation.isPending.value
)
const isConversationPinned = computed(() =>
  Boolean(selectedConversation.value?.metadata?.is_pinned ?? selectedConversation.value?.metadata?.pinned)
)
const isConversationArchived = computed(() =>
  Boolean(selectedConversation.value?.metadata?.is_archived ?? selectedConversation.value?.metadata?.archived)
)
const isConversationMuted = computed(() =>
  Boolean(selectedConversation.value?.metadata?.is_muted ?? selectedConversation.value?.metadata?.muted)
)
const isConversationUnread = computed(() =>
  Boolean(
    selectedConversation.value?.metadata?.is_unread ??
    ((conversationMetaNumber('unread_count') ?? 0) > 0)
  )
)

watch(
  conversations,
  (items) => {
    if (!items.some((conversation) => conversation.provider_chat_id === selectedConversationId.value)) {
      selectedConversationId.value = items[0]?.provider_chat_id ?? ''
    }
  },
  { immediate: true }
)

watch(mediaItems, (items) => {
  if (!items.length) {
    selectedMediaAttachmentId.value = null
    return
  }
  if (
    selectedMediaAttachmentId.value &&
    items.some((item) => item.attachment_id === selectedMediaAttachmentId.value)
  ) {
    return
  }
  selectedMediaAttachmentId.value = firstPreviewableMediaAttachmentId(items)
})

function selectConversation(providerChatId: string) {
  selectedConversationId.value = providerChatId
}

function conversationMetaNumber(key: string): number | null {
  const value = selectedConversation.value?.metadata?.[key]
  return typeof value === 'number' ? value : null
}


function selectMediaItem(item: WhatsappWebMediaItem): void {
  selectedMediaAttachmentId.value = mediaAttachmentId(item)
}

function clearMediaPreview(): void {
  selectedMediaAttachmentId.value = null
}

function requireProviderChatId(message: WhatsAppPanelMessage): string | null {
  const providerChatId = message.provider_chat_id ?? message.conversation_id
  if (providerChatId) return providerChatId
  actionError.value = t('Message is missing provider conversation metadata')
  return null
}

function requireProviderMessageId(message: WhatsAppPanelMessage): string | null {
  const providerMessageId = message.provider_message_id ?? message.provider_record_id
  if (providerMessageId) return providerMessageId
  actionError.value = t('Message is missing provider message metadata')
  return null
}

async function sendMessage() {
  const conversation = selectedConversation.value
  const text = draftText.value.trim()
  if (!conversation || !text || isBusy.value) return
  actionMessage.value = ''
  actionError.value = ''
  try {
    const result = await sendMutation.mutateAsync({
      account_id: conversation.account_id,
      provider_chat_id: conversation.provider_chat_id,
      text,
    })
    draftText.value = ''
    actionMessage.value = `WhatsApp message ${result.status}`
  } catch (error) {
    actionError.value = error instanceof Error ? error.message : String(error)
  }
}

async function replyToMessage(message: WhatsAppPanelMessage) {
  const text = draftText.value.trim()
  if (!text || isBusy.value) return
  actionMessage.value = ''
  actionError.value = ''
  try {
    const result = await replyMutation.mutateAsync({
      message_id: message.message_id,
      text,
    })
    draftText.value = ''
    actionMessage.value = `WhatsApp reply ${result.status}`
  } catch (error) {
    actionError.value = error instanceof Error ? error.message : String(error)
  }
}

function beginEditMessage(message: WhatsAppPanelMessage) {
  if (isBusy.value) return
  editingMessageId.value = message.message_id
  editDraftText.value = message.text ?? message.body_text_preview ?? ''
  actionMessage.value = ''
  actionError.value = ''
}

function cancelEditMessage() {
  editingMessageId.value = null
  editDraftText.value = ''
}

function beginForwardMessage(message: WhatsAppPanelMessage) {
  if (isBusy.value) return
  forwardingMessageId.value = message.message_id
  forwardTargetFilter.value = ''
  forwardTargetConversationId.value =
    conversations.value.find(
      (conversation) => conversation.provider_chat_id !== message.provider_chat_id
    )?.provider_chat_id ?? ''
}

function cancelForwardMessage() {
  forwardingMessageId.value = null
  forwardTargetC
```
_Source file truncated after 12000 characters. / Исходный файл обрезан после 12000 символов._

### `frontend/src/domains/communications/views/CommunicationsEmptyPage.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/views/CommunicationsEmptyPage.vue`
- Size bytes / Размер в байтах: `980`
- Included characters / Включено символов: `980`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { useI18n } from '../../../platform/i18n'
import Icon from '../../../shared/ui/Icon.vue'

const { t } = useI18n()
</script>

<template>
  <div class="empty-section">
    <Icon icon="tabler:messages-off" class="empty-icon" />
    <h3 class="empty-label">{{ t('communications.empty.title') }}</h3>
    <p class="empty-description">
      {{ t('communications.empty.description') }}
    </p>
  </div>
</template>

<style scoped>
.empty-section {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  padding: 2rem;
  color: var(--hh-text-secondary, #6b7280);
  text-align: center;
}

.empty-icon {
  width: 48px;
  height: 48px;
  margin-bottom: 1rem;
  opacity: 0.4;
}

.empty-label {
  font-size: 1.125rem;
  font-weight: 600;
  margin-bottom: 0.5rem;
  color: var(--hh-text-primary, #1f2937);
}

.empty-description {
  font-size: 0.875rem;
  max-width: 280px;
  line-height: 1.4;
}
</style>
```

### `frontend/src/domains/communications/views/CommunicationsPage.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/communications/views/CommunicationsPage.vue`
- Size bytes / Размер в байтах: `11684`
- Included characters / Включено символов: `11684`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import AttachmentSearchPanel from '../components/AttachmentSearchPanel.vue'
import BulkActionsBar from '../components/BulkActionsBar.vue'
import CommunicationsActionBar from '../components/CommunicationsActionBar.vue'
import CommunicationsDetailPane from '../components/CommunicationsDetailPane.vue'
import CommunicationsListPane from '../components/CommunicationsListPane.vue'
import CommunicationsRailPane from '../components/CommunicationsRailPane.vue'
import CommunicationsWorkbench from '../components/CommunicationsWorkbench.vue'
import ComposeDrawer from '../components/ComposeDrawer.vue'
import CommunicationFolderStrip from '../components/CommunicationFolderStrip.vue'
import CommunicationsCallsPanel from '../components/CommunicationsCallsPanel.vue'
import OutboxStatusStrip from '../components/OutboxStatusStrip.vue'
import SavedSearchStrip from '../components/SavedSearchStrip.vue'
import TelegramCommunicationsPanel from '../providers/telegram/views/TelegramCommunicationsPanel.vue'
import WhatsAppCommunicationsPanel from '../providers/whatsapp/views/WhatsAppCommunicationsPanel.vue'
import { communicationSectionTabs } from '../constants/sectionTabs'
import AccountSetupModal from '../../../shared/mailSetup/AccountSetupModal.vue'
import { useNavigationStore } from '../../../shared/stores/navigation'
import { useCommunicationsPageController } from './useCommunicationsPageController'

const nav = useNavigationStore()

const {
  activeFolderId,
  activeSavedSearchId,
  activeSectionId,
  areResourcesLoading,
  blockers,
  clearSyncStatus,
  drafts,
  handleAnalyze,
  handleAddLabel,
  handleBilingualReplySend,
  handleBulkAction,
  handleCreateNote,
  handleCreateTask,
  handleDeleteDraft,
  handleForwardMessage,
  handleMarkMessageRead,
  handleMarkMessageUnread,
  handleDeleteFromProvider,
  handleFolderDeleted,
  handleFolderSelect,
  handleApplyAiReply,
  handleGenerateAiReply,
  handleLoadMoreDrafts,
  handleLoadMoreMessages,
  handleLoadMoreSubscriptions,
  handleLoadMoreThreads,
  handleLoadMoreTopSenders,
  handleMute,
  handleNewMessage,
  handleOpenDraft,
  handleOpenThreadMessage,
  handleReply,
  handleReplyAll,
  handleReplyToThreadMessage,
  handleRedirectMessage,
  handleRemoveLabel,
  handleReviewRecipients,
  handleReviewSecurity,
  handleSavedSearchDeleted,
  handleSavedSearchSelect,
  handleSaveThreadReplyDraft,
  handleSearchQueryUpdate,
  handleSelectMessage,
  handleSelectThread,
  handleSendThreadReply,
  handleSnoozeMessage,
  handleSyncNow,
  handleUpdateSyncSettings,
  handleToggleImportant,
  handleTogglePin,
  handleTranslate,
  handleExportMessage,
  hasMoreDrafts,
  hasMoreOutboxItems,
  hasMoreSubscriptions,
  hasRail,
  hasMoreTopSenders,
  hasThreadNextPage,
  hasVisibleNextPage,
  isAccountSetupOpen,
  isBulkActionRunning,
  isFetchingThreadNextPage,
  isFetchingVisibleNextPage,
  isLoadingMoreDrafts,
  isLoadingMoreOutbox,
  isLoadingMoreSubscriptions,
  isLoadingMoreTopSenders,
  isNavigatorListLoading,
  isOutboxLoading,
  isSelectedThreadLoading,
  isSyncSettingsLoading,
  isSyncSettingsSaving,
  isThreadReplySending,
  isUndoingOutbox,
  loadMoreOutboxItems,
  mailboxHealth,
  messageDetail,
  outboxErrorMessage,
  outboxItems,
  prefetchMoreOutboxItems,
  refetchMailList,
  savedSearchChannelKind,
  selectedBulkCount,
  selectedMailSyncSettings,
  selectedThreadErrorMessage,
  selectedThreadMessages,
  selectSection,
  stateCounts,
  store,
  subscriptions,
  topSenders,
  undoOutbox,
  visibleMailList,
  visibleMailListErrorMessage
} = useCommunicationsPageController()
</script>

<template>
  <section class="communications-page">
    <TelegramCommunicationsPanel v-if="nav.activeCommunicationSection === 'telegram'" />
    <WhatsAppCommunicationsPanel v-else-if="nav.activeCommunicationSection === 'whatsapp'" />
    <CommunicationsCallsPanel
      v-else-if="nav.activeCommunicationSection === 'calls'"
      mode="calls"
    />
    <CommunicationsCallsPanel
      v-else-if="nav.activeCommunicationSection === 'meetings'"
      mode="meetings"
    />
    <template v-else>
    <CommunicationsActionBar
      :search-query="store.messageSearchQuery"
      :section-tabs="communicationSectionTabs"
      :active-section-id="activeSectionId"
      :state-counts="stateCounts"
      :is-sync-busy="store.isMailSyncBusy"
      :sync-status-message="store.mailSyncStatusMessage"
      :sync-error="store.mailSyncError"
      :sync-settings="selectedMailSyncSettings"
      :is-sync-settings-loading="isSyncSettingsLoading"
      :is-sync-settings-saving="isSyncSettingsSaving"
      :health="mailboxHealth"
      :subscriptions="subscriptions"
      :top-senders="topSenders"
      :blockers="blockers"
      :are-resources-loading="areResourcesLoading"
      :has-more-subscriptions="hasMoreSubscriptions"
      :is-loading-more-subscriptions="isLoadingMoreSubscriptions"
      :has-more-top-senders="hasMoreTopSenders"
      :is-loading-more-top-senders="isLoadingMoreTopSenders"
      :drafts="drafts"
      :has-more-drafts="hasMoreDrafts"
      :is-loading-more-drafts="isLoadingMoreDrafts"
      :action-status="store.mailActionStatus"
      :action-error="store.mailActionError"
      :last-message-export="store.lastMessageExport"
      :page-error="store.communicationsError"
      @update:search-query="handleSearchQueryUpdate"
      @search="refetchMailList"
      @open-account-setup="isAccountSetupOpen = true"
      @compose="handleNewMessage"
      @sync-now="handleSyncNow"
      @update-sync-settings="handleUpdateSyncSettings"
      @load-more-subscriptions="handleLoadMoreSubscriptions"
      @load-more-top-senders="handleLoadMoreTopSenders"
      @clear-sync-status="clearSyncStatus"
      @select-section="selectSection"
      @open-draft="handleOpenDraft"
      @delete-draft="handleDeleteDraft"
      @load-more-drafts="handleLoadMoreDrafts"
      @clear-page-error="store.setCommunicationsError('')"
    />

    <CommunicationsWorkbench :is-loading="isNavigatorListLoading" :has-error="Boolean(visibleMailListErrorMessage)" :has-rail="hasRail">
      <template #list>
        <div class="communications-list-stack">
          <BulkActionsBar
            v-if="selectedBulkCount > 0"
            :selected-count="selectedBulkCount"
            :is-running="isBulkActionRunning"
            @action="handleBulkAction"
            @clear="store.clearMessageSelection"
          />
          <OutboxStatusStrip
            :items="outboxItems"
            :is-loading="isOutboxLoading"
            :is-loading-more="isLoadingMoreOutbox"
            :has-more="hasMoreOutboxItems"
            :is-undoing="isUndoingOutbox"
            :error-message="outboxErrorMessage"
            @undo="undoOutbox"
            @prefetch-more="prefetchMoreOutboxItems"
            @load-more="loadMoreOutboxItems"
          />
          <SavedSearchStrip
            :account-id="store.selectedMailAccountId || null"
            :active-id="activeSavedSearchId"
            :current-query="store.messageSearchQuery"
            :current-workflow-state="store.mailStateFilter"
            :current-local-state="store.mailLocalStateFilter"
            :current-channel-kind="savedSearchChannelKind || 'email'"
            @select="handleSavedSearchSelect"
            @deleted="handleSavedSearchDeleted"
          />
          <CommunicationFolderStrip
            :account-id="store.selectedMailAccountId || null"
            :active-id="activeFolderId"
            @select="handleFolderSelect"
            @deleted="handleFolderDeleted"
          />
          <AttachmentSearchPanel :account-id="store.selectedMailAccountId || null" />
          <CommunicationsListPane
            :account-id="store.selectedMailAccountId"
            :messages="visibleMailList"
            :threads="store.threads"
            :selected-index="store.selectedConversationIndex"
            :selected-thread-id="store.selectedThreadId"
            :selected-message-ids="store.selectedMessageIds"
            :navigator-mode="store.communicationsNavigatorMode"
            :is-folder-mode="Boolean(activeFolderId)"
            :is-loading="isNavigatorListLoading"
            :has-next-page="hasVisibleNextPage"
            :is-fetching-next-page="isFetchingVisibleNextPage"
            :has-thread-next-page="hasThreadNextPage"
            :is-fetching-thread-next-page="isFetchingThreadNextPage"
            :error-message="visibleMailListErrorMessage"
            @select="handleSelectMessage"
            @select-thread="handleSelectThread"
            @toggle-selection="store.toggleMessageSelection"
            @select-visible="store.selectVisibleMessages"
            @clear-selection="store.clearMessageSelection"
            @load-more="handleLoadMoreMessages"
            @load-more-threads="handleLoadMoreThreads"
            @update:navigator-mode="store.setNavigatorMode"
          />
        </div>
      </template>

      <template #detail>
        <CommunicationsDetailPane
          :detail="messageDetail"
          :insight="store.mailMessageInsight"
          :active-tab="store.activeMessageContextTab"
          :selected-thread="store.selectedThread"
          :thread-messages="selectedThreadMessages"
          :is-thread-loading="isSelectedThreadLoading"
          :thread-error-message="selectedThreadErrorMessage"
          :is-thread-reply-sending="isThreadReplySending"
          @update:active-tab="store.setActiveMessageContextTab"
          @reply="handleReply"
          @reply-all="handleReplyAll"
          @forward-message="handleForwardMessage"
          @redirect-message="handleRedirectMessage"
          @create-task="handleCreateTask"
          @create-note="handleCreateNote"
          @translate="handleTranslate"
          @generate-ai-reply="handleGenerateAiReply"
          @apply-ai-reply="handleApplyAiReply"
          @review-security="handleReviewSecurity"
          @review-recipients="handleReviewRecipients"
          @analyze="handleAnalyze"
          @send-bilingual-reply="handleBilingualReplySend"
          @mark-message-read="handleMarkMessageRead"
          @mark-message-unread="handleMarkMessageUnread"
          @delete-from-provider="handleDeleteFromProvider"
          @toggle-pin="handleTogglePin"
          @toggle-important="handleToggleImportant"
          @mute="handleMute"
          @export-message="handleExportMessage"
          @add-label="handleAddLabel"
          @remove-label="handleRemoveLabel"
          @snooze-message="handleSnoozeMessage"
          @open-compose="handleNewMessage"
          @open-thread-message="handleOpenThreadMessage"
          @reply-to-thread-message="handleReplyToThreadMessage"
          @save-thread-reply-draft="handleSaveThreadReplyDraft"
          @send-thread-reply="handleSendThreadReply"
        />
      </template>

      <template #rail>
        <CommunicationsRailPane
          :detail="messageDetail"
          :inspector-mode="store.communicationsInspectorMode"
          :projects="store.communicationProjects"
          :tasks="store.communicationTasks"
          @update:inspector-mode="store.setInspectorMode"
        />
      </template>
    </CommunicationsWorkbench>

    <ComposeDrawer v-if="store.isComposeOpen" />
    <AccountSetupModal v-if="isAccountSetupOpen" @close="isAccountSetupOpen = false" />
    </template>
  </section>
</template>

<style scoped>
.communications-page {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
  background: var(--hh-bg-primary, #ffffff);
}

.communications-list-stack {
  height: 100%; min-height: 0; display: flex; flex-direction: column; overflow: hidden;
}
</style>
```

### `frontend/src/domains/documents/components/DocumentsInsights.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/documents/components/DocumentsInsights.vue`
- Size bytes / Размер в байтах: `367`
- Included characters / Включено символов: `367`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { useI18n } from '../../../platform/i18n'

const { t } = useI18n()
</script>

<template>
  <div class="widget-frame">
    <section class="panel info-card">
      <h2>{{ t('Document Insights') }}</h2>
      <p>{{ t('AI analysis results will appear here when document processing is complete.') }}</p>
    </section>
  </div>
</template>
```

### `frontend/src/domains/documents/components/DocumentsList.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/documents/components/DocumentsList.vue`
- Size bytes / Размер в байтах: `3231`
- Included characters / Включено символов: `3231`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { ref, computed } from 'vue'
import { useVirtualizer } from '@tanstack/vue-virtual'
import { useI18n } from '../../../platform/i18n'
import Icon from '../../../shared/ui/Icon.vue'
import type { DocDisplayItem } from '../types/documents'

const { t } = useI18n()

const props = defineProps<{
  documents: DocDisplayItem[]
  searchQuery: string
  activeFilter: string
}>()

const emit = defineEmits<{
  'update:search-query': [value: string]
  'update:active-filter': [value: string]
}>()

const parentRef = ref<HTMLDivElement | null>(null)

const virtualOptions = computed(() => ({
  count: props.documents.length,
  getScrollElement: () => parentRef.value,
  estimateSize: () => 80,
  overscan: 5
}))

const virtualizer = useVirtualizer(virtualOptions)

const virtualItems = computed(() => virtualizer.value.getVirtualItems())
const totalSize = computed(() => virtualizer.value.getTotalSize())
</script>

<template>
  <div class="widget-frame documents-list-panel">
    <div class="document-main-list">
      <div class="document-filter-bar">
        <div class="segmented">
          <button
            type="button"
            :class="['segmented', { active: activeFilter === 'all' }]"
            @click="emit('update:active-filter', 'all')"
          >{{ t('All') }}</button>
          <button
            type="button"
            :class="['segmented', { active: activeFilter === 'shared' }]"
            @click="emit('update:active-filter', 'shared')"
          >{{ t('Shared') }}</button>
          <button
            type="button"
            :class="['segmented', { active: activeFilter === 'recent' }]"
            @click="emit('update:active-filter', 'recent')"
          >{{ t('Recent') }}</button>
        </div>
        <label class="local-search">
          <Icon icon="tabler:search" :size="17" />
          <input
            :placeholder="t('Search documents...')"
            :value="searchQuery"
            @input="emit('update:search-query', ($event.target as HTMLInputElement).value)"
          />
        </label>
      </div>
      <div ref="parentRef" class="documents-scroll-container">
        <div v-if="documents.length === 0" class="muted p-4">{{ t('No documents found') }}</div>
        <div v-else :style="{ height: `${totalSize}px` }">
          <article
            v-for="vitem in virtualItems"
            :key="String(vitem.key)"
            class="document-row"
            :style="{ transform: `translateY(${vitem.start}px)`, height: `${vitem.size}px` }"
          >
            <span :class="['round-icon', documents[vitem.index].tone]">
              <Icon :icon="documents[vitem.index].icon" :size="20" />
            </span>
            <div>
              <strong>{{ documents[vitem.index].name }}</strong>
              <small>{{ documents[vitem.index].source }} &middot; {{ documents[vitem.index].project }} &middot; {{ documents[vitem.index].type }} &middot; {{ documents[vitem.index].size }}</small>
            </div>
            <time>{{ documents[vitem.index].date }}</time>
          </article>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.documents-scroll-container {
  flex: 1;
  overflow-y: auto;
}
</style>
```

### `frontend/src/domains/documents/components/DocumentsNavigation.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/documents/components/DocumentsNavigation.vue`
- Size bytes / Размер в байтах: `1065`
- Included characters / Включено символов: `1065`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { useI18n } from '../../../platform/i18n'
import Icon from '../../../shared/ui/Icon.vue'

const { t } = useI18n()

const smartCollections = [
  'Recently Added 48', 'Recently Opened 24', 'Important 32',
  'Shared with Me 18', 'Requires Review 7', 'Contracts & Legal 23', 'Financial 15'
]
const myFolders = [
  'Макошь', 'Projects', 'Personal', 'Work', 'Archive 2024', 'Clients', 'References'
]
</script>

<template>
  <div class="widget-frame documents-navigation-panel">
    <aside class="left-panels">
      <section class="panel info-card">
        <h2>{{ t('Smart Collections') }}</h2>
        <div v-for="item in smartCollections" :key="item" class="collection-row">
          {{ t(item) }}
        </div>
      </section>
      <section class="panel info-card">
        <h2>{{ t('My Folders') }}</h2>
        <div v-for="folder in myFolders" :key="folder" class="collection-row">
          <Icon icon="tabler:folder" :size="16" />
          {{ t(folder) }}
        </div>
      </section>
    </aside>
  </div>
</template>
```

### `frontend/src/domains/documents/components/DocumentsProcessingJobs.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/documents/components/DocumentsProcessingJobs.vue`
- Size bytes / Размер в байтах: `1349`
- Included characters / Включено символов: `1349`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { useI18n } from '../../../platform/i18n'
import type { DocumentProcessingJob } from '../types/documents'

const { t } = useI18n()

defineProps<{
  jobs: DocumentProcessingJob[]
  isLoading: boolean
  detailError: string
  retryingJobId: string | null
}>()

const emit = defineEmits<{
  retry: [job: DocumentProcessingJob]
}>()
</script>

<template>
  <div class="widget-frame">
    <section class="panel info-card">
      <h2>{{ t('Processing Jobs') }}</h2>
      <div v-if="isLoading" class="graph-strip-message">
        <span>{{ t('Loading jobs.') }}</span>
      </div>
      <template v-else>
        <div v-for="job in jobs.slice(0, 5)" :key="job.job_id" class="job-row">
          <strong>{{ job.document_id }}</strong>
          <span :class="['status-chip', job.status]">{{ job.status }}</span>
          <small>{{ job.step }} &middot; {{ job.queued_at }}</small>
          <button
            v-if="job.status === 'failed'"
            type="button"
            :disabled="retryingJobId === job.document_id"
            @click="emit('retry', job)"
          >
            {{ retryingJobId === job.document_id ? t('Retrying...') : t('Retry') }}
          </button>
        </div>
        <p v-if="detailError" class="inline-error">{{ detailError }}</p>
      </template>
    </section>
  </div>
</template>
```

### `frontend/src/domains/documents/components/DocumentsSourceCards.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/documents/components/DocumentsSourceCards.vue`
- Size bytes / Размер в байтах: `853`
- Included characters / Включено символов: `853`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { useI18n } from '../../../platform/i18n'
import Icon from '../../../shared/ui/Icon.vue'

const { t } = useI18n()

const sources = [
  { name: 'Google Drive', count: '1,243', icon: 'tabler:brand-google-drive' },
  { name: 'OneDrive', count: '812', icon: 'tabler:brand-office' },
  { name: 'Dropbox', count: '342', icon: 'tabler:brand-dropbox' },
  { name: 'Notion', count: '256', icon: 'tabler:brand-notion' }
]
</script>

<template>
  <div class="widget-frame documents-source-cards-panel">
    <div class="document-source-cards">
      <div v-for="src in sources" :key="src.name" class="source-card">
        <span class="round-icon">
          <Icon :icon="src.icon" :size="24" />
        </span>
        <strong>{{ t(src.name) }}</strong>
        <em>{{ src.count }}</em>
      </div>
    </div>
  </div>
</template>
```

### `frontend/src/domains/documents/views/DocumentsPage.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/documents/views/DocumentsPage.vue`
- Size bytes / Размер в байтах: `3043`
- Included characters / Включено символов: `3043`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useI18n } from '../../../platform/i18n'
import Icon from '../../../shared/ui/Icon.vue'
import { useDocumentsStore } from '../stores/documents'
import { useDocumentProcessingJobsQuery } from '../queries/useDocumentsQuery'
import { retryDocumentProcessingJob } from '../api/documents'
import type { DocumentProcessingJob, DocDisplayItem } from '../types/documents'
import DocumentsSourceCards from '../components/DocumentsSourceCards.vue'
import DocumentsNavigation from '../components/DocumentsNavigation.vue'
import DocumentsList from '../components/DocumentsList.vue'
import DocumentsProcessingJobs from '../components/DocumentsProcessingJobs.vue'
import DocumentsInsights from '../components/DocumentsInsights.vue'

const { t } = useI18n()
const store = useDocumentsStore()

const { data: jobsData, isLoading, refetch: refetchJobs } = useDocumentProcessingJobsQuery(50)

const documentProcessingJobs = computed(() => jobsData.value?.items ?? [])

const documents = computed<DocDisplayItem[]>(() =>
  documentProcessingJobs.value.map((job) => ({
    name: `${job.document_id} (${job.step})`,
    source: 'Макошь',
    project: job.status,
    type: job.step,
    date: job.queued_at,
    size: job.last_error_summary || 'No errors',
    icon: 'tabler:file-text',
    tone: job.status === 'succeeded' ? 'green' : job.status === 'failed' ? 'red' : 'amber'
  }))
)

async function handleRetry(job: DocumentProcessingJob) {
  if (store.retryingJobId === job.job_id) return
  store.setRetryingJobId(job.job_id)
  store.setDocumentsError('')
  try {
    await retryDocumentProcessingJob(job.job_id, {
      command_id: `document-processing-retry-${Date.now()}-${job.job_id}`
    })
  } catch (e) {
    store.setDocumentsError(e instanceof Error ? e.message : 'Retry failed')
  }
  await refetchJobs()
  store.setRetryingJobId(null)
}

onMounted(() => {
  refetchJobs()
})
</script>

<template>
  <section class="documents-page">
    <div class="view-header">
      <div class="view-title-with-icon">
        <span class="hero-mark small"><Icon icon="tabler:file-text" :size="28" /></span>
        <div>
          <h1>{{ t('Documents') }}</h1>
          <p>{{ t('All your documents from connected sources') }}</p>
        </div>
      </div>
    </div>
    <div class="documents-layout">
      <DocumentsSourceCards />
      <DocumentsNavigation />
      <DocumentsList
        :documents="documents"
        :search-query="store.searchQuery"
        :active-filter="store.activeFilter"
        @update:search-query="store.setSearchQuery"
        @update:active-filter="store.setActiveFilter"
      />
      <aside class="stacked-rail">
        <DocumentsProcessingJobs
          :jobs="documentProcessingJobs"
          :is-loading="isLoading"
          :detail-error="store.documentsError"
          :retrying-job-id="store.retryingJobId"
          @retry="handleRetry"
        />
        <DocumentsInsights />
      </aside>
    </div>
  </section>
</template>
```

### `frontend/src/domains/home/components/HomeActiveProjects.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/home/components/HomeActiveProjects.vue`
- Size bytes / Размер в байтах: `1499`
- Included characters / Включено символов: `1499`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import Icon from '../../../shared/ui/Icon.vue'
import type { ProjectItem } from '../types/home'

defineProps<{
  projects: ProjectItem[]
}>()

const emit = defineEmits<{
  navigateToProjects: []
}>()
</script>

<template>
  <div class="widget-frame" data-widget-id="home-active-projects">
    <section class="panel full-band">
      <header class="panel-title-row">
        <h2>Active Projects</h2>
        <button type="button" class="link-button" @click="emit('navigateToProjects')">
          View all projects
        </button>
      </header>
      <div class="project-card-row" data-widget-fit-content>
        <template v-if="projects.length > 0">
          <article v-for="project in projects" :key="project.name" class="compact-project">
            <span :class="['round-icon', project.tone]">
              <Icon :icon="project.icon" :size="20" />
            </span>
            <div>
              <strong>{{ project.name }}</strong>
              <small>{{ project.kind }}</small>
            </div>
            <progress class="progress" max="100" :value="project.progress" :aria-label="`${project.name} progress`">
              {{ project.progress }}%
            </progress>
            <em>{{ project.progress }}%</em>
          </article>
        </template>
        <button type="button" class="new-tile" disabled>
          <Icon icon="tabler:plus" :size="22" />
          New Project
        </button>
      </div>
    </section>
  </div>
</template>
```

### `frontend/src/domains/home/components/HomeMetrics.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/home/components/HomeMetrics.vue`
- Size bytes / Размер в байтах: `2149`
- Included characters / Включено символов: `2145`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import Icon from '../../../shared/ui/Icon.vue'
import type { StatCard } from '../types/home'

defineProps<{
  stats: StatCard[]
}>()
</script>

<template>
  <div class="widget-frame" data-widget-id="home-metrics">
    <div class="metric-grid home-metrics" data-widget-fit-content>
      <article v-for="metric in stats" :key="metric.label" class="metric-card">
        <span>{{ metric.label }}</span>
        <div>
          <strong>{{ metric.value }}</strong>
          <Icon :icon="metric.icon" :size="26" />
        </div>
        <small>↑ {{ metric.delta }}</small>
      </article>
      <article class="metric-card focus-card">
        <span>Focus Score</span>
        <div class="score-ring"><strong>78</strong></div>
        <small>Good ↑ 5</small>
      </article>
    </div>
  </div>
</template>

<style scoped>
.home-metrics {
  grid-template-columns: repeat(6, minmax(110px, 1fr));
}

.metric-grid {
  display: grid;
  gap: 12px;
  padding: 18px;
}

.metric-card {
  background: var(--hh-surface-panel);
  border: 1px solid var(--hh-border);
  border-radius: var(--hh-radius-md);
  padding: 16px;
  display: grid;
  gap: 10px;
  min-height: 100px;
}

.metric-card span {
  color: var(--hh-color-text-muted);
  font-size: 12px;
  font-weight: 500;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.metric-card > div {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}

.metric-card strong {
  font-size: 28px;
  font-weight: 700;
  color: var(--hh-color-text-bright);
}

.metric-card small {
  color: var(--hh-color-accent);
  font-size: 12px;
  font-weight: 500;
}

.focus-card {
  background: linear-gradient(135deg, rgba(45, 235, 204, 0.08), rgba(42, 139, 167, 0.12));
  border-color: rgba(45, 235, 204, 0.25);
}

.score-ring {
  display: grid;
  place-items: center;
  width: 48px;
  height: 48px;
  border: 4px solid rgba(45, 235, 204, 0.82);
  border-right-color: rgba(42, 139, 167, 0.8);
  border-bottom-color: rgba(236, 183, 70, 0.8);
  border-radius: var(--hh-radius-round);
  margin: 0 !important;
}

.score-ring strong {
  font-size: 14px;
}
</style>
```

### `frontend/src/domains/home/components/HomePeopleTalked.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/home/components/HomePeopleTalked.vue`
- Size bytes / Размер в байтах: `936`
- Included characters / Включено символов: `936`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import Icon from '../../../shared/ui/Icon.vue'
import type { PersonItem } from '../types/home'

defineProps<{
  people: PersonItem[]
}>()
</script>

<template>
  <div class="widget-frame" data-widget-id="home-people-talked-to">
    <section class="panel mini-panel" data-widget-fit-content>
      <header class="panel-title-row">
        <h2>People You Talked To</h2>
        <button type="button" class="link-button" disabled>View all</button>
      </header>
      <div class="person-list">
        <article v-for="person in people" :key="person.name">
          <span class="round-icon ghost">
            <Icon icon="tabler:user" :size="20" />
          </span>
          <span>
            <strong>{{ person.name }}</strong>
            <small>{{ person.meta }}</small>
          </span>
          <Icon :icon="person.icon" :size="18" />
        </article>
      </div>
    </section>
  </div>
</template>
```

### `frontend/src/domains/home/components/HomePriorities.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/home/components/HomePriorities.vue`
- Size bytes / Размер в байтах: `954`
- Included characters / Включено символов: `953`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import type { TaskItem } from '../types/home'

defineProps<{
  tasks: TaskItem[]
}>()
</script>

<template>
  <div class="widget-frame" data-widget-id="home-priorities">
    <section class="panel priorities-panel" data-widget-fit-content>
      <header class="panel-title-row">
        <div>
          <h2>Today's Priorities</h2>
          <p>Focus on what matters most</p>
        </div>
      </header>
      <div class="task-stack">
        <label v-for="task in tasks.slice(0, 5)" :key="task.title">
          <input type="checkbox" />
          <span>
            <strong>{{ task.title }}</strong>
            <small>{{ task.assignee }} · {{ task.due }}</small>
          </span>
          <em :class="{ high: task.priority === 'High' }">{{ task.priority }}</em>
        </label>
      </div>
      <button type="button" class="link-row" disabled>
        View all tasks
      </button>
    </section>
  </div>
</template>
```

### `frontend/src/domains/home/components/HomeSystemStatus.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/home/components/HomeSystemStatus.vue`
- Size bytes / Размер в байтах: `651`
- Included characters / Включено символов: `651`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
defineProps<{
  statusError: string
}>()
</script>

<template>
  <div class="widget-frame" data-widget-id="home-system-status">
    <section class="panel mini-panel" data-widget-fit-content>
      <header class="panel-title-row">
        <h2>System Status</h2>
      </header>
      <ul class="status-list">
        <li>All systems operational</li>
        <li>AI Agents online <span>5/5</span></li>
        <li>Data synchronized <span>2m ago</span></li>
        <li>Local AI models <span>Ready</span></li>
      </ul>
      <p v-if="statusError" class="inline-error">{{ statusError }}</p>
    </section>
  </div>
</template>
```

### `frontend/src/domains/home/components/HomeUpcoming.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/home/components/HomeUpcoming.vue`
- Size bytes / Размер в байтах: `927`
- Included characters / Включено символов: `927`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import Icon from '../../../shared/ui/Icon.vue'
</script>

<template>
  <div class="widget-frame" data-widget-id="home-upcoming">
    <section class="panel schedule-panel" data-widget-fit-content>
      <header class="panel-title-row">
        <div>
          <h2>Upcoming</h2>
          <p>Your schedule</p>
        </div>
      </header>
      <div class="schedule-list">
        <article>
          <time>Today</time>
          <strong>14:00 Call with Acme Corp</strong>
          <span>16:30 Review Q2 Report</span>
        </article>
        <article>
          <time>Tomorrow</time>
          <strong>10:00 Project Макошь - Planning</strong>
          <span>15:00 Design Review</span>
        </article>
      </div>
      <button type="button" class="link-row" disabled>
        View full calendar <Icon icon="tabler:arrow-right" :size="15" />
      </button>
    </section>
  </div>
</template>
```

### `frontend/src/domains/home/components/HomeWhatsNew.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/home/components/HomeWhatsNew.vue`
- Size bytes / Размер в байтах: `1205`
- Included characters / Включено символов: `1205`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import Icon from '../../../shared/ui/Icon.vue'
import type { FeedItem } from '../types/home'

defineProps<{
  items: FeedItem[]
}>()
</script>

<template>
  <div class="widget-frame" data-widget-id="home-whats-new">
    <section class="panel feed-panel" data-widget-fit-content>
      <header class="panel-title-row">
        <div>
          <h2>What's New</h2>
          <p>Key changes and important updates</p>
        </div>
        <button type="button" class="ghost-button" disabled>All Types</button>
      </header>
      <div class="feed-list">
        <article v-for="item in items" :key="item.title + item.time" class="feed-row">
          <span :class="['round-icon', item.tone ?? '']">
            <Icon :icon="item.icon" :size="22" />
          </span>
          <div>
            <strong>{{ item.title }}</strong>
            <p>{{ item.meta }}</p>
            <em v-if="item.tag">{{ item.tag }}</em>
          </div>
          <time>{{ item.time }}</time>
        </article>
      </div>
      <button type="button" class="link-row" disabled>
        View all events <Icon icon="tabler:arrow-right" :size="15" />
      </button>
    </section>
  </div>
</template>
```

### `frontend/src/domains/home/views/HomePage.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/home/views/HomePage.vue`
- Size bytes / Размер в байтах: `4254`
- Included characters / Включено символов: `4250`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { computed } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from '../../../platform/i18n'
import HomeMetrics from '../components/HomeMetrics.vue'
import HomeWhatsNew from '../components/HomeWhatsNew.vue'
import HomePriorities from '../components/HomePriorities.vue'
import HomeUpcoming from '../components/HomeUpcoming.vue'
import HomePeopleTalked from '../components/HomePeopleTalked.vue'
import HomeSystemStatus from '../components/HomeSystemStatus.vue'
import HomeActiveProjects from '../components/HomeActiveProjects.vue'
import { useCommunicationMessagesQuery, useMailboxHealthQuery } from '../queries/useHomeQuery'
import type { StatCard, FeedItem, PersonItem, ProjectItem, TaskItem } from '../types/home'

const { t } = useI18n()
const router = useRouter()

const { data: messages } = useCommunicationMessagesQuery(50)
const { data: mailboxHealth } = useMailboxHealthQuery()

const channelIcons: Record<string, string> = {
  email: 'tabler:mail',
  gmail: 'tabler:brand-gmail',
  icloud: 'tabler:cloud',
  imap: 'tabler:server',
  telegram_user: 'tabler:brand-telegram',
  telegram_bot: 'tabler:brand-telegram',
  whatsapp_web: 'tabler:brand-whatsapp'
}

const homeStats = computed<StatCard[]>(() => {
  const stats: StatCard[] = []
  if (mailboxHealth.value) {
    stats.push({ label: t('Messages'), value: String(mailboxHealth.value.total_messages), delta: `+${mailboxHealth.value.unread}`, icon: 'tabler:mail' })
    stats.push({ label: t('Needs attention'), value: String(mailboxHealth.value.needs_action), delta: `+${mailboxHealth.value.important}`, icon: 'tabler:alert-circle' })
    stats.push({ label: t('Waiting'), value: String(mailboxHealth.value.waiting), delta: `${mailboxHealth.value.done} ${t('done')}`, icon: 'tabler:message-reply' })
  }
  stats.push({ label: t('Projects'), value: '—', delta: t('active'), icon: 'tabler:briefcase' })
  stats.push({ label: t('Persons'), value: '—', delta: t('enriched'), icon: 'tabler:user-plus' })
  return stats
})

const whatsNew = computed<FeedItem[]>(() => {
  const items: FeedItem[] = []
  const msgs = messages.value ?? []
  for (const msg of msgs.slice(0, 5)) {
    const sender = msg.sender_display_name || msg.sender || t('Unknown')
    items.push({
      icon: channelIcons[msg.channel_kind] || 'tabler:message',
      title: t('New message from {sender}').replace('{sender}', sender),
      meta: msg.subject || msg.body_text_preview,
      time: msg.occurred_at || msg.projected_at,
      tone: 'blue'
    })
  }
  return items
})

const peopleTalked = computed<PersonItem[]>(() => {
  const seen = new Set<string>()
  const result: PersonItem[] = []
  const msgs = messages.value ?? []
  for (const msg of msgs) {
    const sender = msg.sender_display_name || msg.sender || t('Unknown')
    if (seen.has(sender)) continue
    seen.add(sender)
    result.push({
      name: sender,
      meta: msg.subject || msg.body_text_preview,
      icon: 'tabler:message'
    })
    if (result.length >= 5) break
  }
  return result
})

function navigateToProjects() {
  router.push({ name: 'projects' })
}
</script>

<template>
  <section class="home-page">
    <div class="hero-row">
      <HomeMetrics :stats="homeStats" />
    </div>

    <div class="dashboard-grid">
      <HomeWhatsNew :items="whatsNew" />
      <HomePriorities :tasks="[] as TaskItem[]" />
      <HomeUpcoming />

      <aside class="stacked-rail">
        <HomePeopleTalked :people="peopleTalked" />
        <HomeSystemStatus statusError="" />
      </aside>
    </div>

    <HomeActiveProjects :projects="[] as ProjectItem[]" @navigate-to-projects="navigateToProjects" />
  </section>
</template>

<style scoped>
.home-page {
  padding: 18px;
  display: grid;
  gap: 14px;
}

.hero-row {
  --hh-zone-rows: 3;
  display: grid;
  grid-template-columns: 300px minmax(640px, 1fr);
  align-items: center;
  gap: 14px;
  min-height: var(--hh-widget-card);
}

.dashboard-grid {
  display: grid;
  grid-template-columns: 1fr 280px;
  gap: 14px;
}

.stacked-rail {
  display: grid;
  gap: 14px;
  align-content: start;
}

@media (max-width: 1359px) {
  .hero-row {
    grid-template-columns: 1fr;
  }
  .dashboard-grid {
    grid-template-columns: 1fr;
  }
}
</style>
```

### `frontend/src/domains/knowledge/components/KnowledgeGraphCanvas.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/knowledge/components/KnowledgeGraphCanvas.vue`
- Size bytes / Размер в байтах: `4449`
- Included characters / Включено символов: `4449`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { useKnowledgeStore } from '../stores/knowledge'
import { graphNodeKindIcon } from '../stores/knowledge'

const store = useKnowledgeStore()
</script>

<template>
  <div class="graph-canvas">
    <template v-if="store.graphError && !store.graphNeighborhood">
      <div class="state-card error-card">
        <Icon icon="tabler:alert-circle" class="state-icon error-icon" />
        <p>{{ store.graphError }}</p>
      </div>
    </template>

    <template v-else-if="!store.graphSummary">
      <div class="state-card loading-card">
        <Icon icon="tabler:loader-2" class="state-icon spin" />
        <p>Loading graph summary...</p>
      </div>
    </template>

    <template v-else-if="store.graphSummary.is_empty">
      <div class="state-card empty-card">
        <Icon icon="tabler:graph" class="state-icon" />
        <p>No knowledge graph projection yet</p>
      </div>
    </template>

    <template v-else-if="store.graphNeighborhood && store.graphCanvasNodes.length > 0">
      <svg viewBox="0 0 100 100" preserveAspectRatio="none" class="graph-svg">
        <line
          v-for="(edge, i) in store.graphCanvasEdges"
          :key="'edge-' + i"
          :x1="edge.x1" :y1="edge.y1"
          :x2="edge.x2" :y2="edge.y2"
          :class="['graph-edge', edge.review_state === 'suggested' ? 'edge-suggested' : '']"
        />
        <text
          v-for="(edge, i) in store.graphCanvasEdges"
          :key="'edge-label-' + i"
          :x="(edge.x1 + edge.x2) / 2"
          :y="(edge.y1 + edge.y2) / 2"
          class="graph-edge-label"
        >{{ edge.label }}</text>
      </svg>
      <div class="graph-nodes-layer">
        <button
          v-for="node in store.graphCanvasNodes"
          :key="node.node_id"
          :class="[
            'graph-node-btn',
            'kind-' + node.node_kind,
            node.isSelected ? 'selected' : ''
          ]"
          :style="{ left: node.x + '%', top: node.y + '%' }"
          @click="store.selectGraphNode({ node_id: node.node_id, node_kind: node.node_kind, label: node.label, properties: {}, stable_key: '', created_at: '', updated_at: '' } as any)"
        >
          <Icon :icon="graphNodeKindIcon(node.node_kind)" class="node-icon" />
          <span class="node-label">{{ node.label }}</span>
        </button>
      </div>
    </template>

    <template v-else>
      <div class="state-card idle-card">
        <Icon icon="tabler:pointer" class="state-icon" />
        <p>Select a node from search or filter to explore</p>
      </div>
    </template>
  </div>
</template>

<style scoped>
.graph-canvas {
  position: relative;
  width: 100%;
  height: 100%;
  min-height: 400px;
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: hidden;
}

.graph-svg {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
  pointer-events: none;
}

.graph-edge {
  stroke: hsl(var(--border));
  stroke-width: 0.3;
}

.edge-suggested {
  stroke: hsl(var(--warning));
  stroke-dasharray: 1, 1;
}

.graph-edge-label {
  font-size: 2px;
  fill: hsl(var(--muted-foreground));
  text-anchor: middle;
  dominant-baseline: middle;
}

.graph-nodes-layer {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
}

.graph-node-btn {
  position: absolute;
  transform: translate(-50%, -50%);
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 2px;
  background: hsl(var(--card));
  border: 1px solid hsl(var(--border));
  border-radius: 8px;
  padding: 4px 8px;
  cursor: pointer;
  transition: box-shadow 0.15s, border-color 0.15s;
  max-width: 80px;
}

.graph-node-btn:hover {
  box-shadow: 0 2px 8px hsl(var(--shadow) / 0.15);
  border-color: hsl(var(--ring));
}

.graph-node-btn.selected {
  border-color: hsl(var(--ring));
  box-shadow: 0 0 0 2px hsl(var(--ring) / 0.3);
}

.node-icon {
  font-size: 18px;
  color: hsl(var(--foreground));
}

.node-label {
  font-size: 6px;
  color: hsl(var(--muted-foreground));
  text-align: center;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  max-width: 70px;
}

.state-card {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  padding: 24px;
  color: hsl(var(--muted-foreground));
}

.state-icon {
  font-size: 32px;
}

.spin {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.error-icon {
  color: hsl(var(--destructive));
}
</style>
```

### `frontend/src/domains/knowledge/components/KnowledgeNodeInspector.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/knowledge/components/KnowledgeNodeInspector.vue`
- Size bytes / Размер в байтах: `4912`
- Included characters / Включено символов: `4911`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { useKnowledgeStore, graphNodeKindLabel } from '../stores/knowledge'
import Icon from '../../../shared/ui/Icon.vue'

const store = useKnowledgeStore()
</script>

<template>
  <div class="inspector-panels">
    <!-- Selected Node -->
    <div class="inspector-section" v-if="store.selectedGraphNode">
      <h4 class="section-title">Selected Node</h4>
      <div class="property-list">
        <div class="property-row">
          <span class="prop-key">Kind</span>
          <span class="prop-value">{{ graphNodeKindLabel(store.selectedGraphNode.node_kind) }}</span>
        </div>
        <div class="property-row">
          <span class="prop-key">Label</span>
          <span class="prop-value">{{ store.selectedGraphNode.label }}</span>
        </div>
        <div class="property-row" v-if="store.selectedGraphNode.stable_key">
          <span class="prop-key">Stable Key</span>
          <span class="prop-value mono">{{ store.selectedGraphNode.stable_key }}</span>
        </div>
        <div
          v-for="prop in store.selectedGraphProperties"
          :key="prop.key"
          class="property-row"
        >
          <span class="prop-key">{{ prop.key }}</span>
          <span class="prop-value">{{ String(prop.value) }}</span>
        </div>
        <div class="property-row">
          <span class="prop-key">Created</span>
          <span class="prop-value">{{ new Date(store.selectedGraphNode.created_at).toLocaleDateString() }}</span>
        </div>
        <div class="property-row">
          <span class="prop-key">Updated</span>
          <span class="prop-value">{{ new Date(store.selectedGraphNode.updated_at).toLocaleDateString() }}</span>
        </div>
      </div>
    </div>

    <!-- Connections -->
    <div class="inspector-section" v-if="store.graphNeighborCounts.length > 0">
      <h4 class="section-title">Connections</h4>
      <div class="property-list">
        <div
          v-for="nc in store.graphNeighborCounts"
          :key="nc.kind"
          class="property-row"
        >
          <span class="prop-key">{{ graphNodeKindLabel(nc.kind) }}</span>
          <span class="prop-value count">{{ nc.count }}</span>
        </div>
      </div>
    </div>

    <!-- Evidence -->
    <div class="inspector-section" v-if="store.graphNeighborhood && store.graphNeighborhood.evidence.length > 0">
      <h4 class="section-title">Evidence</h4>
      <div
        v-for="ev in store.graphNeighborhood.evidence.slice(0, 5)"
        :key="ev.edge_id"
        class="evidence-item"
      >
        <p class="evidence-excerpt">{{ ev.excerpt || 'No excerpt' }}</p>
        <p class="evidence-meta">{{ ev.source_kind }} · {{ ev.source_id }}</p>
      </div>
    </div>

    <!-- Graph Statistics -->
    <div class="inspector-section" v-if="store.graphSummary">
      <h4 class="section-title">Graph Statistics</h4>
      <div class="property-list">
        <div class="property-row">
          <span class="prop-key">Nodes</span>
          <span class="prop-value count">
            {{ store.graphSummary.node_counts.reduce((acc, c) => acc + c.count, 0) }}
          </span>
        </div>
        <div class="property-row">
          <span class="prop-key">Connections</span>
          <span class="prop-value count">
            {{ store.graphSummary.edge_counts.reduce((acc, c) => acc + c.count, 0) }}
          </span>
        </div>
        <div class="property-row">
          <span class="prop-key">Evidence</span>
          <span class="prop-value count">{{ store.graphSummary.evidence_count }}</span>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.inspector-panels {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.inspector-section {
  background: hsl(var(--card));
  border: 1px solid hsl(var(--border));
  border-radius: 8px;
  padding: 12px;
}

.section-title {
  font-size: 13px;
  font-weight: 600;
  margin: 0 0 8px;
  color: hsl(var(--foreground));
}

.property-list {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.property-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 8px;
}

.prop-key {
  font-size: 12px;
  color: hsl(var(--muted-foreground));
  white-space: nowrap;
}

.prop-value {
  font-size: 12px;
  color: hsl(var(--foreground));
  text-align: right;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  max-width: 180px;
}

.prop-value.mono {
  font-family: ui-monospace, monospace;
  font-size: 11px;
}

.prop-value.count {
  font-weight: 600;
}

.evidence-item {
  padding: 8px 0;
  border-bottom: 1px solid hsl(var(--border));
}

.evidence-item:last-child {
  border-bottom: none;
}

.evidence-excerpt {
  font-size: 12px;
  color: hsl(var(--foreground));
  margin: 0 0 4px;
  line-height: 1.4;
}

.evidence-meta {
  font-size: 11px;
  color: hsl(var(--muted-foreground));
  margin: 0;
}
</style>
```

### `frontend/src/domains/knowledge/components/KnowledgePolygraphReview.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/knowledge/components/KnowledgePolygraphReview.vue`
- Size bytes / Размер в байтах: `5826`
- Included characters / Включено символов: `5826`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { useKnowledgeStore, contradictionSeverityTone, formatContradictionClaim, formatContradictionTime, formatContradictionSource } from '../stores/knowledge'
import Icon from '../../../shared/ui/Icon.vue'

const store = useKnowledgeStore()

const props = defineProps<{
  observations: import('../types/knowledge').ContradictionObservation[]
  error: string
  loading: boolean
}>()

function severityClass(severity: import('../types/knowledge').ContradictionSeverity): string {
  return `severity-${severity}`
}

async function handleReview(
  observation: import('../types/knowledge').ContradictionObservation,
  reviewState: Exclude<import('../types/knowledge').ContradictionReviewState, 'suggested'>
) {
  await store.reviewContradictionObservation(observation, reviewState)
}
</script>

<template>
  <div class="polygraph-review">
    <div v-if="error" class="state-line">
      <Icon icon="tabler:alert-circle" class="state-icon error-icon" />
      <span>{{ error }}</span>
    </div>

    <div v-else-if="loading" class="state-line">
      <Icon icon="tabler:loader-2" class="state-icon spin" />
      <span>Loading contradiction observations...</span>
    </div>

    <div v-else-if="observations.length === 0" class="state-line">
      <Icon icon="tabler:check-circle" class="state-icon success-icon" />
      <span>No contradictions detected</span>
    </div>

    <div v-else class="observations-list">
      <div
        v-for="obs in observations"
        :key="obs.observation_id"
        :class="['observation-card', severityClass(obs.severity)]"
      >
        <div class="obs-header">
          <span :class="['severity-badge', 'badge-' + obs.severity]">{{ obs.severity }}</span>
          <span class="obs-time">{{ formatContradictionTime(obs.created_at) }}</span>
        </div>
        <p class="obs-claim">{{ formatContradictionClaim(obs) }}</p>
        <p class="obs-source">
          {{ formatContradictionSource(obs.old_source_kind, obs.old_source_id) }}
        </p>
        <p class="obs-source">
          {{ formatContradictionSource(obs.new_source_kind, obs.new_source_id) }}
        </p>
        <div v-if="obs.review_state === 'suggested'" class="obs-actions">
          <button
            type="button"
            class="action-btn confirm-btn"
            :disabled="store.reviewingContradictionObservationId === obs.observation_id"
            @click="handleReview(obs, 'user_confirmed')"
          >
            <Icon icon="tabler:check" />
            Confirm
          </button>
          <button
            type="button"
            class="action-btn reject-btn"
            :disabled="store.reviewingContradictionObservationId === obs.observation_id"
            @click="handleReview(obs, 'user_rejected')"
          >
            <Icon icon="tabler:x" />
            Reject
          </button>
        </div>
        <div v-else class="obs-reviewed">
          <span class="review-badge">{{ obs.review_state }}</span>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.polygraph-review {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.state-line {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 12px;
  color: hsl(var(--muted-foreground));
  font-size: 13px;
}

.state-icon {
  font-size: 18px;
  flex-shrink: 0;
}

.spin {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.error-icon { color: hsl(var(--destructive)); }
.success-icon { color: hsl(var(--success)); }

.observations-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.observation-card {
  background: hsl(var(--card));
  border: 1px solid hsl(var(--border));
  border-radius: 8px;
  padding: 12px;
}

.observation-card.severity-critical {
  border-left: 3px solid hsl(var(--destructive));
}

.observation-card.severity-high {
  border-left: 3px solid hsl(var(--warning));
}

.observation-card.severity-medium {
  border-left: 3px solid hsl(var(--primary));
}

.observation-card.severity-low {
  border-left: 3px solid hsl(var(--muted));
}

.obs-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 8px;
}

.severity-badge {
  font-size: 10px;
  font-weight: 600;
  text-transform: uppercase;
  padding: 2px 6px;
  border-radius: 4px;
}

.badge-critical { background: hsl(var(--destructive) / 0.15); color: hsl(var(--destructive)); }
.badge-high { background: hsl(var(--warning) / 0.15); color: hsl(var(--warning)); }
.badge-medium { background: hsl(var(--primary) / 0.15); color: hsl(var(--primary)); }
.badge-low { background: hsl(var(--muted) / 0.3); color: hsl(var(--muted-foreground)); }

.obs-time {
  font-size: 11px;
  color: hsl(var(--muted-foreground));
  margin-left: auto;
}

.obs-claim {
  font-size: 13px;
  color: hsl(var(--foreground));
  margin: 0 0 6px;
  line-height: 1.4;
}

.obs-source {
  font-size: 11px;
  color: hsl(var(--muted-foreground));
  margin: 0 0 2px;
}

.obs-actions {
  display: flex;
  gap: 8px;
  margin-top: 10px;
}

.action-btn {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 4px 12px;
  font-size: 12px;
  border-radius: 6px;
  border: 1px solid hsl(var(--border));
  background: hsl(var(--card));
  cursor: pointer;
  transition: background 0.15s;
}

.action-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.confirm-btn:hover:not(:disabled) {
  background: hsl(var(--success) / 0.1);
  border-color: hsl(var(--success));
}

.reject-btn:hover:not(:disabled) {
  background: hsl(var(--destructive) / 0.1);
  border-color: hsl(var(--destructive));
}

.obs-reviewed {
  margin-top: 8px;
}

.review-badge {
  font-size: 11px;
  font-weight: 500;
  padding: 2px 8px;
  border-radius: 4px;
  background: hsl(var(--muted) / 0.3);
  color: hsl(var(--muted-foreground));
}
</style>
```

### `frontend/src/domains/knowledge/views/KnowledgePage.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/knowledge/views/KnowledgePage.vue`
- Size bytes / Размер в байтах: `9355`
- Included characters / Включено символов: `9355`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { ref, computed, watch, onMounted } from 'vue'
import { useKnowledgeStore, graphNodeKindIcon, graphNodeKindLabel } from '../stores/knowledge'
import { useGraphSummaryQuery, useContradictionsQuery } from '../queries/useKnowledgeQuery'
import KnowledgeGraphCanvas from '../components/KnowledgeGraphCanvas.vue'
import KnowledgeNodeInspector from '../components/KnowledgeNodeInspector.vue'
import KnowledgePolygraphReview from '../components/KnowledgePolygraphReview.vue'
import Icon from '../../../shared/ui/Icon.vue'
import type { GraphNode } from '../types/knowledge'

const store = useKnowledgeStore()

const { data: summaryData, error: summaryError, isLoading: summaryLoading } = useGraphSummaryQuery()
const { data: contradictionsData, error: contradictionsError, isLoading: contradictionsLoading } = useContradictionsQuery(50)

onMounted(() => {
  if (summaryData.value) {
    store.setGraphSummary(summaryData.value, '')
  }
})

watch(summaryData, (val) => {
  if (val) {
    store.setGraphSummary(val, '')
  }
})
watch(summaryError, (err) => {
  if (err) {
    store.setGraphSummary(null, (err as Error)?.message || 'Unknown error')
  }
})
watch(contradictionsData, (val) => {
  if (val) {
    store.setContradictionObservations(val)
  }
})

const searchQuery = ref('')
const searchLoading = ref(false)

async function handleSearch() {
  if (!searchQuery.value.trim()) return
  searchLoading.value = true
  try {
    await store.runGraphSearch(searchQuery.value)
  } finally {
    searchLoading.value = false
  }
}

async function handleSelectSearchResult(node: GraphNode) {
  searchQuery.value = ''
  store.setGraphSearchResults([], '')
  await store.selectGraphNode(node)
}

const suggestedContradictionsCount = computed(() => {
  return store.contradictionObservations.filter((o) => o.review_state === 'suggested').length
})

async function loadGraphNodeChoices() {
  await store.loadGraphNodeChoices()
}
</script>

<template>
  <div class="knowledge-page">
    <!-- Filter tabs -->
    <div class="filter-tabs" v-if="store.graphFilterChips.length > 0">
      <button
        v-for="chip in store.graphFilterChips"
        :key="chip.kind"
        class="filter-chip"
        @click="loadGraphNodeChoices"
      >
        <Icon :icon="chip.icon" class="chip-icon" />
        <span class="chip-label">{{ chip.label }}</span>
        <span class="chip-count">{{ chip.count }}</span>
      </button>
      <button class="filter-chip rebuild-btn" @click="loadGraphNodeChoices">
        <Icon icon="tabler:refresh" class="chip-icon" />
        <span>Rebuild</span>
      </button>
    </div>

    <!-- Loading state -->
    <div v-if="summaryLoading" class="loading-banner">
      <Icon icon="tabler:loader-2" class="spin" />
      <span>Loading knowledge graph...</span>
    </div>

    <!-- Error state -->
    <div v-else-if="store.graphError && !store.graphSummary" class="error-banner">
      <Icon icon="tabler:alert-circle" />
      <span>{{ store.graphError }}</span>
    </div>

    <!-- Main layout -->
    <div v-else class="knowledge-layout">
      <!-- Workbench -->
      <div class="graph-workbench">
        <!-- Toolbar -->
        <div class="workbench-toolbar">
          <form class="search-form" @submit.prevent="handleSearch">
            <input
              v-model="searchQuery"
              type="text"
              placeholder="Search graph nodes..."
              class="search-input"
            />
            <button
              type="submit"
              class="primary-button"
              :disabled="searchLoading || !searchQuery.trim()"
            >
              <Icon icon="tabler:search" />
              Search
            </button>
          </form>
        </div>

        <!-- Search results -->
        <div v-if="store.graphSearchResults.length > 0" class="search-results">
          <div
            v-for="node in store.graphSearchResults"
            :key="node.node_id"
            class="search-result-item"
            @click="handleSelectSearchResult(node)"
          >
            <Icon :icon="graphNodeKindIcon(node.node_kind)" class="result-icon" />
            <div class="result-info">
              <span class="result-label">{{ node.label }}</span>
              <span class="result-kind">{{ graphNodeKindLabel(node.node_kind) }}</span>
            </div>
          </div>
        </div>

        <!-- Graph canvas -->
        <KnowledgeGraphCanvas />

        <!-- Status bar -->
        <div class="status-bar" v-if="store.graphSummary">
          <span>{{ store.graphSummary.node_counts.length }} node types</span>
          <span>{{ store.graphSummary.edge_counts.length }} edge types</span>
          <span>{{ store.graphSummary.evidence_count }} evidence items</span>
        </div>
      </div>

      <!-- Side rail -->
      <div class="knowledge-side-rail">
        <!-- Polygraph Review -->
        <div class="rail-section">
          <h3 class="rail-section-title">
            <Icon icon="tabler:git-compare" />
            Polygraph Review
            <span v-if="suggestedContradictionsCount > 0" class="section-badge">
              {{ suggestedContradictionsCount }}
            </span>
          </h3>
          <KnowledgePolygraphReview
            :observations="store.contradictionObservations"
            :error="(contradictionsError as Error)?.message || ''"
            :loading="contradictionsLoading"
          />
        </div>

        <!-- Node Inspector -->
        <div class="rail-section">
          <h3 class="rail-section-title">
            <Icon icon="tabler:info-circle" />
            Node Inspector
          </h3>
          <KnowledgeNodeInspector />
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.knowledge-page {
  display: flex;
  flex-direction: column;
  gap: 8px;
  height: 100%;
  padding: 16px;
}

.filter-tabs {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.filter-chip {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 4px 10px;
  font-size: 12px;
  border-radius: 999px;
  border: 1px solid hsl(var(--border));
  background: hsl(var(--card));
  cursor: pointer;
  transition: background 0.15s;
}

.filter-chip:hover {
  background: hsl(var(--accent));
}

.chip-icon {
  font-size: 14px;
  color: hsl(var(--muted-foreground));
}

.chip-label {
  color: hsl(var(--foreground));
}

.chip-count {
  font-size: 11px;
  color: hsl(var(--muted-foreground));
  background: hsl(var(--muted) / 0.3);
  padding: 0 6px;
  border-radius: 999px;
}

.rebuild-btn {
  margin-left: auto;
}

.loading-banner,
.error-banner {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 12px 16px;
  font-size: 14px;
}

.loading-banner {
  color: hsl(var(--muted-foreground));
}

.error-banner {
  color: hsl(var(--destructive));
}

.spin {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.knowledge-layout {
  display: flex;
  gap: 12px;
  flex: 1;
  min-height: 0;
}

.graph-workbench {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 8px;
  min-width: 0;
}

.workbench-toolbar {
  display: flex;
  align-items: center;
  gap: 8px;
}

.search-form {
  display: flex;
  gap: 6px;
  flex: 1;
}

.search-input {
  flex: 1;
  padding: 6px 12px;
  font-size: 13px;
  border: 1px solid hsl(var(--border));
  border-radius: 6px;
  background: hsl(var(--background));
  color: hsl(var(--foreground));
  outline: none;
}

.search-input:focus {
  border-color: hsl(var(--ring));
}

.primary-button {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 6px 14px;
  font-size: 13px;
  border-radius: 6px;
  border: none;
  background: hsl(var(--primary));
  color: hsl(var(--primary-foreground));
  cursor: pointer;
}

.primary-button:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.search-results {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  padding: 8px;
  background: hsl(var(--card));
  border: 1px solid hsl(var(--border));
  border-radius: 8px;
}

.search-result-item {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 4px 8px;
  border-radius: 6px;
  cursor: pointer;
  transition: background 0.15s;
}

.search-result-item:hover {
  background: hsl(var(--accent));
}

.result-icon {
  font-size: 14px;
  color: hsl(var(--muted-foreground));
}

.result-info {
  display: flex;
  flex-direction: column;
}

.result-label {
  font-size: 12px;
  font-weight: 500;
  color: hsl(var(--foreground));
}

.result-kind {
  font-size: 10px;
  color: hsl(var(--muted-foreground));
}

.status-bar {
  display: flex;
  gap: 16px;
  padding: 6px 12px;
  font-size: 11px;
  color: hsl(var(--muted-foreground));
  background: hsl(var(--card));
  border: 1px solid hsl(var(--border));
  border-radius: 6px;
}

.knowledge-side-rail {
  width: 320px;
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  gap: 12px;
  overflow-y: auto;
}

.rail-section {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.rail-section-title {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 14px;
  font-weight: 600;
  margin: 0;
  color: hsl(var(--foreground));
}

.section-badge {
  font-size: 11px;
  font-weight: 600;
  padding: 1px 7px;
  border-radius: 999px;
  background: hsl(var(--destructive) / 0.15);
  color: hsl(var(--destructive));
}
</style>
```

### `frontend/src/domains/notes/components/NotesInsights.vue`

- Resolved path / Полный путь: `/Users/avm/projects/Personal/makosh/frontend/src/domains/notes/components/NotesInsights.vue`
- Size bytes / Размер в байтах: `363`
- Included characters / Включено символов: `363`
- Truncated / Обрезано: `no`

```text
<script setup lang="ts">
import { useI18n } from '../../../platform/i18n'

const { t } = useI18n()
</script>

<template>
  <div class="widget-frame">
    <section class="panel info-card">
      <h2>{{ t('Note Insights') }}</h2>
      <p>{{ t('AI generated summaries and connections across your notes will appear here.') }}</p>
    </section>
  </div>
</template>
```
