<script setup lang="ts">
import { computed } from 'vue'
import type { ClientModuleBootstrapV1 } from '../../../gen/makosh/gateway/v1/client_bootstrap_pb'
import Icon from '../../../shared/ui/Icon.vue'
import { useMailAccountPortability } from '../portability/useMailAccountPortability'
import './mailPortabilityPanel.css'

const props = defineProps<{ module: ClientModuleBootstrapV1 | null }>()
const portability = useMailAccountPortability(() => props.module)
const gmailAuthorizationUrl = computed(
	() => portability.importState.value?.gmailOAuthStarted?.authorizationUrl ?? '',
)
</script>

<template>
	<section class="mail-portability-panel">
		<header class="mail-portability-panel__header">
			<span><Icon icon="tabler:transfer" /></span>
			<div>
				<small>Account portability</small>
				<h3>Move configuration without moving credentials</h3>
				<p>
					Export uses fresh owner proof. Import records every Settings, Vault and Mail
					receipt separately so partial progress stays visible and resumable.
				</p>
			</div>
		</header>

		<div class="mail-portability-panel__grid">
			<section class="mail-portability-card">
				<header>
					<strong>Export</strong>
					<span>Typed MailAccountExportV1</span>
				</header>
				<button
					type="button"
					:disabled="!portability.canExport.value || portability.busy.value"
					@click="portability.prepareExport"
				>
					<Icon icon="tabler:shield-check" />
					Authorize and prepare
				</button>
				<textarea
					v-if="portability.exportJson.value"
					:value="portability.exportJson.value"
					readonly
					aria-label="Sanitized Mail account export"
				/>
				<button
					v-if="portability.exportJson.value"
					type="button"
					class="secondary"
					@click="portability.downloadExport"
				>
					<Icon icon="tabler:download" />
					Download JSON
				</button>
			</section>

			<section class="mail-portability-card">
				<header>
					<strong>Import</strong>
					<span>Explicit multi-authority workflow</span>
				</header>
				<label>
					<span>Sanitized export JSON</span>
					<textarea
						v-model="portability.importJson.value"
						placeholder="Paste MailAccountExportV1 JSON"
					/>
				</label>
				<button
					type="button"
					:disabled="portability.busy.value"
					@click="portability.startImport"
				>
					<Icon icon="tabler:file-check" />
					Validate and apply configuration
				</button>

				<template v-if="portability.importState.value?.imap">
					<label>
						<span>IMAP password</span>
						<input
							v-model="portability.imapPassword.value"
							type="password"
							autocomplete="new-password"
						>
					</label>
				</template>
				<template v-if="portability.importState.value?.smtp">
					<label>
						<span>SMTP password</span>
						<input
							v-model="portability.smtpPassword.value"
							type="password"
							autocomplete="new-password"
						>
					</label>
				</template>
				<button
					v-if="portability.importState.value?.configurationApplyReceipt"
					type="button"
					:disabled="portability.busy.value"
					@click="portability.continueImport"
				>
					<Icon icon="tabler:player-track-next" />
					Continue next explicit step
				</button>
			</section>
		</div>

		<section
			v-if="portability.importState.value"
			class="mail-portability-progress"
			aria-live="polite"
		>
			<header>
				<strong>Import receipts</strong>
				<span>{{ portability.importState.value.phase }}</span>
			</header>
			<ol>
				<li
					v-for="step in portability.steps.value"
					:key="step.label"
					:class="{ complete: step.complete }"
				>
					<Icon :icon="step.complete ? 'tabler:circle-check' : 'tabler:circle-dashed'" />
					{{ step.label }}
				</li>
			</ol>

			<div
				v-if="portability.importState.value.gmailOAuthStarted"
				class="mail-portability-oauth"
			>
				<a
					:href="gmailAuthorizationUrl"
					target="_blank"
					rel="noreferrer"
				>
					Open Gmail authorization
					<Icon icon="tabler:external-link" />
				</a>
				<label>
					<span>Returned state</span>
					<input v-model="portability.gmailState.value" autocomplete="off">
				</label>
				<label>
					<span>Authorization code</span>
					<input
						v-model="portability.gmailAuthorizationCode.value"
						type="password"
						autocomplete="one-time-code"
					>
				</label>
				<button
					type="button"
					:disabled="portability.busy.value"
					@click="portability.completeGmail"
				>
					Complete Gmail OAuth
				</button>
			</div>

			<footer>
				<span v-if="portability.errorCode.value">
					<Icon icon="tabler:alert-triangle" />
					{{ portability.errorCode.value }}
				</span>
				<button
					type="button"
					class="secondary"
					:disabled="portability.busy.value"
					@click="portability.reconcile"
				>
					<Icon icon="tabler:refresh" />
					Reconcile receipts
				</button>
			</footer>
		</section>
	</section>
</template>
