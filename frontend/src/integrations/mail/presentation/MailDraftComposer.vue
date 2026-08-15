<script setup lang="ts">
import { Icon } from '@/shared/ui'
import { MailCompositionModeV1 } from '../../../gen/makosh/mail/composition/v1/client_pb'
import type {
	MailCompositionModel,
	MailDraftEditorPatch,
} from './mailCompositionModel'

defineProps<{
	model: MailCompositionModel
	canDeliver: boolean
	deliveryBusy: boolean
}>()

const emit = defineEmits<{
	deliver: []
	newDraft: []
	removeDraft: []
	saveDraft: []
	selectDraft: [draftId: string]
	updateDraft: [patch: MailDraftEditorPatch]
}>()
</script>

<template>
	<section class="mail-compose-editor">
		<div class="mail-compose-editor__drafts">
			<label for="mail-compose-saved-draft">
				Saved draft
				<select
					id="mail-compose-saved-draft"
					:value="model.draft.draftId"
					:disabled="model.busyAction !== null"
					@change="emit('selectDraft', ($event.target as HTMLSelectElement).value)"
				>
					<option value="">Unsaved draft</option>
					<option v-for="draft in model.drafts" :key="draft.id" :value="draft.id">
						{{ draft.label }} — {{ draft.detail }}
					</option>
				</select>
			</label>
			<button type="button" class="mail-button-secondary" @click="emit('newDraft')">
				<Icon icon="tabler:file-plus" /> New draft
			</button>
		</div>

		<div class="mail-compose-editor__envelope">
			<label for="mail-compose-to">
				<span>To</span>
				<textarea
					id="mail-compose-to"
					rows="1"
					placeholder="name@example.com"
					:value="model.draft.toRecipients"
					@input="emit('updateDraft', { toRecipients: ($event.target as HTMLTextAreaElement).value })"
				/>
			</label>
			<div class="mail-compose-editor__copy-fields">
				<label for="mail-compose-cc">
					<span>Cc</span>
					<input
						id="mail-compose-cc"
						:value="model.draft.ccRecipients"
						@input="emit('updateDraft', { ccRecipients: ($event.target as HTMLInputElement).value })"
					>
				</label>
				<label for="mail-compose-bcc">
					<span>Bcc</span>
					<input
						id="mail-compose-bcc"
						:value="model.draft.bccRecipients"
						@input="emit('updateDraft', { bccRecipients: ($event.target as HTMLInputElement).value })"
					>
				</label>
			</div>
			<label for="mail-compose-subject">
				<span>Subject</span>
				<input
					id="mail-compose-subject"
					autocomplete="off"
					placeholder="Subject"
					:value="model.draft.subject"
					@input="emit('updateDraft', { subject: ($event.target as HTMLInputElement).value })"
				>
			</label>
		</div>

		<label class="mail-compose-editor__body" for="mail-compose-message">
			<span>Message</span>
			<textarea
				id="mail-compose-message"
				rows="12"
				placeholder="Write your message…"
				:value="model.draft.textBody"
				@input="emit('updateDraft', { textBody: ($event.target as HTMLTextAreaElement).value })"
			/>
		</label>

		<details class="mail-compose-editor__delivery-details">
			<summary>Threading and delivery details</summary>
			<div class="mail-composition-inline-fields">
				<label>
					Mode
					<select
						:value="model.draft.mode"
						@change="emit('updateDraft', { mode: Number(($event.target as HTMLSelectElement).value) })"
					>
						<option :value="MailCompositionModeV1.MAIL_COMPOSITION_MODE_NEW">New</option>
						<option :value="MailCompositionModeV1.MAIL_COMPOSITION_MODE_REPLY">Reply</option>
						<option :value="MailCompositionModeV1.MAIL_COMPOSITION_MODE_REPLY_ALL">Reply all</option>
						<option :value="MailCompositionModeV1.MAIL_COMPOSITION_MODE_FORWARD">Forward</option>
						<option :value="MailCompositionModeV1.MAIL_COMPOSITION_MODE_REDIRECT">Redirect</option>
					</select>
				</label>
				<label>Revision <input readonly :value="model.draft.revision || 'new'"></label>
			</div>
			<label>
				Provider conversation ID
				<input
					autocomplete="off"
					:value="model.draft.providerConversationId"
					@input="emit('updateDraft', { providerConversationId: ($event.target as HTMLInputElement).value })"
				>
			</label>
			<label>
				In-reply-to provider message ID
				<input
					autocomplete="off"
					:value="model.draft.inReplyToProviderMessageId"
					@input="emit('updateDraft', { inReplyToProviderMessageId: ($event.target as HTMLInputElement).value })"
				>
			</label>
		</details>

		<div class="mail-composition-actions">
			<button
				type="button"
				:disabled="!model.canMutate || model.busyAction !== null"
				@click="emit('saveDraft')"
			>
				<Icon icon="tabler:device-floppy" />
				{{ model.busyAction === 'draft' ? 'Saving…' : 'Save draft' }}
			</button>
			<button
				v-if="model.draft.draftId"
				type="button"
				class="mail-button-danger"
				:disabled="!model.canMutate || model.busyAction !== null"
				@click="emit('removeDraft')"
			>
				Delete
			</button>
			<button
				type="button"
				:disabled="!canDeliver
					|| deliveryBusy
					|| !model.draft.toRecipients.trim()
					|| !model.draft.textBody.trim()"
				@click="emit('deliver')"
			>
				<Icon icon="tabler:send" />
				{{ deliveryBusy ? 'Sending…' : 'Send' }}
			</button>
		</div>
	</section>
</template>
