import { create, toBinary } from '@bufbuild/protobuf'
import type { Meta, StoryObj } from '@storybook/vue3-vite'
import { http, HttpResponse } from 'msw'
import type { Component } from 'vue'

import AppSettingsPage from '../../src/app/settings/AppSettingsPage.vue'
import { compiledClientSurfaceAdapterIds } from '../../src/app/client-surfaces/compiledClientSurfaceAdapters'
import {
	ClientModuleBootstrapV1Schema,
	ClientModuleSettingsBootstrapV1Schema,
	ClientSettingValueEntryV1Schema,
	ClientSettingValueV1Schema,
	ClientSettingsApplyStateV1,
} from '../../src/gen/makosh/gateway/v1/client_bootstrap_pb'
import {
	MailAccountCatalogV1Schema,
	MailAccountReadinessV1,
	MailAccountStatusV1Schema,
	MailConnectorProfileV1,
	MailProviderPathReadinessV1,
} from '../../src/gen/makosh/mail/account/v1/client_pb'
import {
	recoveryClientBootstrap,
	type ClientBootstrapSnapshot,
} from '../../src/platform/gateway/clientBootstrap'

const meta = {
	title: 'Макошь App/Settings/Clean Room',
	parameters: { layout: 'fullscreen' },
} satisfies Meta

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
	render: () => createSettingsStory('system'),
}

export const Mail: Story = {
	render: () => createSettingsStory('mail'),
	parameters: {
		msw: {
			handlers: [
				http.post(
					'*/makosh.mail.account.v1.MailAccountCatalogService/List',
					() => new HttpResponse(
						toBinary(MailAccountCatalogV1Schema, mailAccountCatalog()),
						{ headers: { 'content-type': 'application/proto' } },
					),
				),
			],
		},
	},
}

function createSettingsStory(initialOwner: 'mail' | 'system'): Component {
	return {
		components: { AppSettingsPage },
		setup() {
			return {
				bootstrap: settingsBootstrap(),
				compiledAdapterIds: compiledClientSurfaceAdapterIds,
				initialOwner,
				languageOptions: [
					{ value: 'en', label: 'English' },
					{ value: 'ru', label: 'Русский' },
				],
			}
		},
		template: `
			<AppSettingsPage
				:bootstrap="bootstrap"
				:compiled-adapter-ids="compiledAdapterIds"
				current-language="en"
				:developer-mode="true"
				:initial-owner="initialOwner"
				:language-options="languageOptions"
			/>
		`,
	}
}

function settingsBootstrap(): ClientBootstrapSnapshot {
	const recovery = recoveryClientBootstrap()
	const mail = create(ClientModuleBootstrapV1Schema, {
		registrationId: 'mail.owner.local',
		moduleId: 'makosh-mail-runtime',
		grantEpoch: 4n,
		capabilityIds: ['mail.delivery.v1', 'mail.sync.v1'],
		sectionsEnabled: true,
		settings: create(ClientModuleSettingsBootstrapV1Schema, {
			schemaMajor: 1,
			schemaRevision: 3,
			desiredRevision: 8n,
			effectiveRevision: 8n,
			applyState: ClientSettingsApplyStateV1.CURRENT,
			values: [
				setting('sync_interval', 'Sync interval', {
					case: 'durationMillis',
					value: 300000n,
				}),
				setting('content_egress', 'Content egress', {
					case: 'booleanValue',
					value: false,
				}),
			],
		}),
	})
	const providerModules = [
		mail,
		...[
			['telegram', 'makosh-telegram-runtime'],
			['whatsapp', 'makosh-whatsapp-runtime'],
			['zulip', 'makosh-zulip-runtime'],
		].map(([provider, moduleId]) => create(ClientModuleBootstrapV1Schema, {
			registrationId: `${provider}.owner.local`,
			moduleId,
			grantEpoch: 2n,
			sectionsEnabled: true,
		})),
	]
	return Object.assign(new Map(recovery), {
		modules: providerModules,
		systemStatus: recovery.systemStatus,
	}) as ClientBootstrapSnapshot
}

function setting(
	settingId: string,
	displayName: string,
	value: { case: 'durationMillis'; value: bigint } | { case: 'booleanValue'; value: boolean },
) {
	return create(ClientSettingValueEntryV1Schema, {
		settingId,
		displayName,
		editable: true,
		value: create(ClientSettingValueV1Schema, { value }),
	})
}

function mailAccountCatalog() {
	return create(MailAccountCatalogV1Schema, {
		accounts: [
			mailAccount('icloud-primary', MailConnectorProfileV1.MAIL_CONNECTOR_PROFILE_IMAP_SMTP),
			mailAccount('gmail-primary', MailConnectorProfileV1.MAIL_CONNECTOR_PROFILE_GMAIL),
		],
	})
}

function mailAccount(connectionId: string, connectorProfile: MailConnectorProfileV1) {
	return create(MailAccountStatusV1Schema, {
		connectionId,
		settingsRevision: 8n,
		runtimeGeneration: 5n,
		readiness: MailAccountReadinessV1.MAIL_ACCOUNT_READINESS_READY,
		connectorProfile,
		syncReadiness: MailProviderPathReadinessV1.MAIL_PROVIDER_PATH_READINESS_READY,
		deliveryReadiness: MailProviderPathReadinessV1.MAIL_PROVIDER_PATH_READINESS_READY,
		lifecycleRevision: 3n,
		configurationInstanceId: connectionId,
	})
}
