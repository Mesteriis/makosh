<script setup lang="ts">
import type {
	MailCompositionModel,
	MailDraftEditorPatch,
	MailSignatureEditorPatch,
	MailTemplateEditorPatch,
} from './mailCompositionModel'
import MailDraftComposer from './MailDraftComposer.vue'
import MailSignatureLibrary from './MailSignatureLibrary.vue'
import MailTemplateLibrary from './MailTemplateLibrary.vue'
import './mailCompositionPanel.css'

defineProps<{
	model: MailCompositionModel
	canDeliver: boolean
	deliveryBusy: boolean
}>()

const emit = defineEmits<{
	applyTemplate: []
	deliver: []
	newDraft: []
	newSignature: []
	newTemplate: []
	refresh: []
	removeDraft: []
	removeSignature: []
	removeTemplate: []
	saveDraft: []
	saveSignature: []
	saveTemplate: []
	selectConnection: [connectionId: string]
	selectDraft: [draftId: string]
	selectSignature: [signatureId: string]
	selectTemplate: [templateId: string]
	updateDraft: [patch: MailDraftEditorPatch]
	updateSignature: [patch: MailSignatureEditorPatch]
	updateTemplate: [patch: MailTemplateEditorPatch]
	useSignature: [signatureId: string]
}>()
</script>

<template>
	<section class="mail-composition-workspace">
		<header class="mail-composition-toolbar">
			<div>
				<span>From</span>
				<p>Choose the provider account used for this message.</p>
			</div>
			<label>
				Connection
				<select
					:value="model.selectedConnectionId"
					:disabled="model.busyAction !== null"
					@change="emit('selectConnection', ($event.target as HTMLSelectElement).value)"
				>
					<option v-if="model.connections.length === 0" value="">Unavailable</option>
					<option v-for="connection in model.connections" :key="connection.id" :value="connection.id">
						{{ connection.label }}
					</option>
				</select>
			</label>
			<button
				type="button"
				:disabled="!model.canQuery || model.busyAction !== null"
				@click="emit('refresh')"
			>
				{{ model.busyAction === 'refresh' ? 'Loading…' : 'Refresh' }}
			</button>
		</header>

		<p v-if="model.statusMessage" class="mail-operational-empty" role="status">
			{{ model.statusMessage }}
		</p>

		<div class="mail-composition-grid">
			<MailDraftComposer
				:model="model"
				:can-deliver="canDeliver"
				:delivery-busy="deliveryBusy"
				@deliver="emit('deliver')"
				@new-draft="emit('newDraft')"
				@remove-draft="emit('removeDraft')"
				@save-draft="emit('saveDraft')"
				@select-draft="emit('selectDraft', $event)"
				@update-draft="emit('updateDraft', $event)"
			/>
		</div>
		<details class="mail-composition-resources">
			<summary>Templates and signatures</summary>
			<div class="mail-composition-library">
				<MailTemplateLibrary
					:model="model"
					@apply-template="emit('applyTemplate')"
					@new-template="emit('newTemplate')"
					@remove-template="emit('removeTemplate')"
					@save-template="emit('saveTemplate')"
					@select-template="emit('selectTemplate', $event)"
					@update-template="emit('updateTemplate', $event)"
				/>
				<MailSignatureLibrary
					:model="model"
					@new-signature="emit('newSignature')"
					@remove-signature="emit('removeSignature')"
					@save-signature="emit('saveSignature')"
					@select-signature="emit('selectSignature', $event)"
					@update-signature="emit('updateSignature', $event)"
					@use-signature="emit('useSignature', $event)"
				/>
			</div>
		</details>
		<p v-if="model.notice" class="mail-operational-notice" role="status">{{ model.notice }}</p>
	</section>
</template>
