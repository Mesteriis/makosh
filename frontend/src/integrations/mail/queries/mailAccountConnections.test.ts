import { create } from '@bufbuild/protobuf'
import { describe, expect, it } from 'vitest'

import {
	ClientModuleBootstrapV1Schema,
	ClientModuleSettingsBootstrapV1Schema,
} from '../../../gen/makosh/gateway/v1/client_bootstrap_pb'
import {
	MailAccountReadinessV1,
	MailAccountStatusV1Schema,
	MailProviderPathReadinessV1,
} from '../../../gen/makosh/mail/account/v1/client_pb'
import {
	mailAccountConnectionFingerprint,
	mailAccountConnections,
} from './mailAccountConnections'

describe('Mail provider account connection discovery', () => {
	it('uses the Mail-owned catalog while compatibility settings remain blocked', () => {
		const modules = [create(ClientModuleBootstrapV1Schema, {
			registrationId: 'mail-registration',
			moduleId: 'makosh-mail-runtime',
			sectionsEnabled: true,
			capabilityIds: ['mail.account.catalog.query.v1'],
			settings: create(ClientModuleSettingsBootstrapV1Schema, {
				desiredRevision: 1n,
				effectiveRevision: 0n,
			}),
		})]
		const accounts = [
			account('secondary', MailAccountReadinessV1.MAIL_ACCOUNT_READINESS_READY),
			account('configuration-only',
				MailAccountReadinessV1.MAIL_ACCOUNT_READINESS_CONFIGURATION_ONLY),
			account('primary', MailAccountReadinessV1.MAIL_ACCOUNT_READINESS_DEGRADED),
			account('deleted', MailAccountReadinessV1.MAIL_ACCOUNT_READINESS_DELETED),
		]

		const connections = mailAccountConnections(modules, accounts)
		expect(connections).toEqual([
			{
				connectionId: 'configuration-only',
				deliveryReady: false,
				registrationId: 'mail-registration',
				syncReady: false,
			},
			{
				connectionId: 'primary',
				deliveryReady: true,
				registrationId: 'mail-registration',
				syncReady: true,
			},
			{
				connectionId: 'secondary',
				deliveryReady: true,
				registrationId: 'mail-registration',
				syncReady: true,
			},
		])
		expect(mailAccountConnectionFingerprint(connections)).toBe(
			'mail-registration:configuration-only|mail-registration:primary|mail-registration:secondary',
		)
	})

	it('fails closed without the provider catalog capability', () => {
		expect(mailAccountConnections([], [
			account('primary', MailAccountReadinessV1.MAIL_ACCOUNT_READINESS_READY),
		])).toEqual([])
	})
})

function account(connectionId: string, readiness: MailAccountReadinessV1) {
	const ready = readiness === MailAccountReadinessV1.MAIL_ACCOUNT_READINESS_READY
		|| readiness === MailAccountReadinessV1.MAIL_ACCOUNT_READINESS_DEGRADED
	return create(MailAccountStatusV1Schema, {
		connectionId,
		readiness,
		syncReadiness: ready
			? MailProviderPathReadinessV1.MAIL_PROVIDER_PATH_READINESS_READY
			: MailProviderPathReadinessV1.MAIL_PROVIDER_PATH_READINESS_CREDENTIAL_REQUIRED,
		deliveryReadiness: ready
			? MailProviderPathReadinessV1.MAIL_PROVIDER_PATH_READINESS_READY
			: MailProviderPathReadinessV1.MAIL_PROVIDER_PATH_READINESS_CREDENTIAL_REQUIRED,
	})
}
