<script setup lang="ts">
import type { ClientModuleBootstrapV1 } from '../../../gen/makosh/gateway/v1/client_bootstrap_pb'
import { MailContactsSyncDirectionV1 } from '../../../gen/makosh/mail_contacts_sync/v1/sync_pb'
import {
	mailContactsSyncStateLabel,
	type MailContactsSyncAccountChoiceV1,
	useMailContactsSyncSettings,
} from '../queries/useMailContactsSyncSettings'
import './mailContactsSyncSettingsPanel.css'

const props = defineProps<{
	module: ClientModuleBootstrapV1 | null
	accounts: readonly MailContactsSyncAccountChoiceV1[]
}>()
const sync = useMailContactsSyncSettings({
	module: () => props.module,
	accounts: () => props.accounts,
})
</script>

<template>
	<section class="mail-contacts-sync-card">
		<header>
			<div>
				<small>Workflow</small>
				<h3>Mail contacts sync</h3>
				<p>Mail publishes provider observations; Contacts accepts only its own commands.</p>
			</div>
			<strong>{{ sync.admitted.value ? 'Admitted' : 'Unavailable' }}</strong>
		</header>
		<div v-if="!sync.admitted.value" class="mail-contacts-sync-skeleton" aria-label="Mail Contacts Sync unavailable">
			<span /><span /><span />
		</div>
		<template v-else>
			<div class="mail-contacts-sync-fields">
				<label>
					<span>Mail account</span>
					<select v-model="sync.accountId.value" :disabled="sync.busy.value || accounts.length === 0">
						<option v-if="accounts.length === 0" value="">No accounts</option>
						<option v-for="account in accounts" :key="account.accountId" :value="account.accountId">
							{{ account.accountId }}
						</option>
					</select>
				</label>
				<label>
					<span>Direction</span>
					<select v-model="sync.direction.value" :disabled="sync.busy.value">
						<option :value="MailContactsSyncDirectionV1.MAIL_CONTACTS_SYNC_DIRECTION_PROVIDER_TO_CONTACTS">Mail → Contacts</option>
						<option :value="MailContactsSyncDirectionV1.MAIL_CONTACTS_SYNC_DIRECTION_BIDIRECTIONAL">Bidirectional</option>
					</select>
				</label>
				<label>
					<span>Schedule interval, seconds</span>
					<input v-model.number="sync.intervalSeconds.value" type="number" min="300" max="604800" :disabled="sync.busy.value" />
				</label>
			</div>
			<div class="mail-contacts-sync-actions">
				<button type="button" :disabled="sync.busy.value || !sync.accountId.value" @click="sync.configure">Apply configuration</button>
				<button type="button" :disabled="!sync.canStart.value" @click="sync.start">Sync now</button>
			</div>
			<dl>
				<div><dt>Configuration</dt><dd>{{ sync.activeTargetId.value || 'Not configured' }}</dd></div>
				<div><dt>State</dt><dd>{{ mailContactsSyncStateLabel(sync.status.value?.state) }}</dd></div>
				<div><dt>Contacts created</dt><dd>{{ sync.status.value?.contactsCreated ?? 0 }}</dd></div>
				<div><dt>Contacts updated</dt><dd>{{ sync.status.value?.contactsUpdated ?? 0 }}</dd></div>
			</dl>
			<p v-if="sync.message.value" class="mail-contacts-sync-message" role="status">{{ sync.message.value }}</p>
		</template>
	</section>
</template>
