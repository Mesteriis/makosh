<script setup lang="ts">
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
	<section class="mail-operational-card mail-composition-draft">
		<div class="mail-card-heading">
			<div>
				<span>Mail-owned draft</span>
				<h2>Compose</h2>
				<p>Draft state is persisted separately from asynchronous provider delivery.</p>
			</div>
			<button type="button" class="mail-button-secondary" @click="emit('newDraft')">New</button>
		</div>

		<label>
			Saved drafts
			<select
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

		<div class="mail-composition-inline-fields">
			<label>
				Mode
				<select
					:value="model.draft.mode"
					@change="emit('updateDraft', {
						mode: Number(($event.target as HTMLSelectElement).value),
					})"
				>
					<option :value="MailCompositionModeV1.MAIL_COMPOSITION_MODE_NEW">New</option>
					<option :value="MailCompositionModeV1.MAIL_COMPOSITION_MODE_REPLY">Reply</option>
					<option :value="MailCompositionModeV1.MAIL_COMPOSITION_MODE_REPLY_ALL">Reply all</option>
					<option :value="MailCompositionModeV1.MAIL_COMPOSITION_MODE_FORWARD">Forward</option>
					<option :value="MailCompositionModeV1.MAIL_COMPOSITION_MODE_REDIRECT">Redirect</option>
				</select>
			</label>
			<label>
				Revision
				<input readonly :value="model.draft.revision || 'new'">
			</label>
		</div>

		<label>
			To <small>One address per line, comma or semicolon</small>
			<textarea
				rows="2"
				:value="model.draft.toRecipients"
				@input="emit('updateDraft', {
					toRecipients: ($event.target as HTMLTextAreaElement).value,
				})"
			/>
		</label>
		<div class="mail-composition-inline-fields">
			<label>
				Cc
				<textarea
					rows="2"
					:value="model.draft.ccRecipients"
					@input="emit('updateDraft', {
						ccRecipients: ($event.target as HTMLTextAreaElement).value,
					})"
				/>
			</label>
			<label>
				Bcc
				<textarea
					rows="2"
					:value="model.draft.bccRecipients"
					@input="emit('updateDraft', {
						bccRecipients: ($event.target as HTMLTextAreaElement).value,
					})"
				/>
			</label>
		</div>
		<label>
			Provider conversation ID <small>Required for provider reply threading</small>
			<input
				autocomplete="off"
				:value="model.draft.providerConversationId"
				@input="emit('updateDraft', {
					providerConversationId: ($event.target as HTMLInputElement).value,
				})"
			>
		</label>
		<label>
			In-reply-to provider message ID
			<input
				autocomplete="off"
				:value="model.draft.inReplyToProviderMessageId"
				@input="emit('updateDraft', {
					inReplyToProviderMessageId: ($event.target as HTMLInputElement).value,
				})"
			>
		</label>
		<label>
			Subject
			<input
				autocomplete="off"
				:value="model.draft.subject"
				@input="emit('updateDraft', {
					subject: ($event.target as HTMLInputElement).value,
				})"
			>
		</label>
		<label>
			Message
			<textarea
				rows="10"
				:value="model.draft.textBody"
				@input="emit('updateDraft', {
					textBody: ($event.target as HTMLTextAreaElement).value,
				})"
			/>
		</label>
		<div class="mail-composition-actions">
			<button
				type="button"
				:disabled="!model.canMutate || model.busyAction !== null"
				@click="emit('saveDraft')"
			>
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
				{{ deliveryBusy ? 'Sending…' : 'Send current draft' }}
			</button>
		</div>
	</section>
</template>
