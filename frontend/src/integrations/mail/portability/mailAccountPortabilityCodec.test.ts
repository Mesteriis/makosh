import { create } from '@bufbuild/protobuf'
import { describe, expect, it } from 'vitest'

import {
	MailAccountReadinessV1,
	MailAccountStatusV1Schema,
	MailConnectorProfileV1,
	MailProviderPathReadinessV1,
} from '../../../gen/makosh/mail/account/v1/client_pb'
import {
	ExportEffectiveOwnerModuleSettingsReceiptV1Schema,
	OwnerSettingEntryV1Schema,
	OwnerSettingValueV1Schema,
} from '../../../gen/makosh/gateway/v1/owner_module_settings_pb'
import { MailAddressBookProviderV1 } from '../../../gen/makosh/mail/portability/v1/portability_pb'
import {
	buildMailAccountExportV1,
	MAIL_SETTINGS_SCHEMA_REVISION_V2,
	mailAccountExportSettingsInputs,
	parseMailAccountExportV1,
	serializeMailAccountExportV1,
} from './mailAccountPortabilityCodec'

describe('Mail account portability codec', () => {
	it('builds and round-trips one typed non-secret IMAP/SMTP export', () => {
		const settings = create(ExportEffectiveOwnerModuleSettingsReceiptV1Schema, {
			registrationId: 'mail-registration',
			schemaMajor: 2,
			schemaRevision: MAIL_SETTINGS_SCHEMA_REVISION_V2,
			effectiveRevision: 7n,
			values: [
				stringSetting('mail.address_book.provider', 'none'),
				stringSetting('mail.connection_id', 'mail-account'),
				stringSetting('mail.inbound.kind', 'imap'),
				stringSetting('mail.imap.host', 'imap.example.test'),
				unsignedSetting('mail.imap.port', 993n),
				stringSetting('mail.imap.username', 'owner@example.test'),
				unsignedSetting('mail.sync.window', 100n),
				unsignedSetting('mail.sync.windows', 2n),
				booleanSetting('mail.smtp.enabled', true),
				stringSetting('mail.smtp.host', 'smtp.example.test'),
				unsignedSetting('mail.smtp.port', 465n),
				stringSetting('mail.smtp.username', 'owner@example.test'),
				stringSetting('mail.smtp.from_address', 'owner@example.test'),
			],
		})
		const status = create(MailAccountStatusV1Schema, {
			connectionId: 'mail-account',
			settingsRevision: 7n,
			runtimeGeneration: 9n,
			readiness: MailAccountReadinessV1.MAIL_ACCOUNT_READINESS_READY,
			connectorProfile: MailConnectorProfileV1.MAIL_CONNECTOR_PROFILE_IMAP_SMTP,
			syncReadiness: MailProviderPathReadinessV1.MAIL_PROVIDER_PATH_READINESS_READY,
			deliveryReadiness: MailProviderPathReadinessV1.MAIL_PROVIDER_PATH_READINESS_READY,
		})

		const exported = buildMailAccountExportV1(settings, status, 10n)
		const json = serializeMailAccountExportV1(exported)
		const parsed = parseMailAccountExportV1(json)
		const desired = mailAccountExportSettingsInputs(parsed)

		expect(parsed).toMatchObject({
			major: 1,
			sourceRegistrationId: 'mail-registration',
			settingsSchemaMajor: 2,
			settingsSchemaRevision: MAIL_SETTINGS_SCHEMA_REVISION_V2,
			effectiveSettingsRevision: 7n,
			configuration: {
				connectionId: 'mail-account',
				inbound: {
					case: 'imap',
					value: {
						host: 'imap.example.test',
						port: 993,
					},
				},
				smtp: {
					host: 'smtp.example.test',
					port: 465,
				},
			},
		})
		expect(json).not.toMatch(/password|credential|secret|token|cursor/i)
		const desiredIds = desired.map(({ settingId }) => settingId)
		expect(desiredIds).toEqual([...desiredIds].sort())
		expect(desired).toContainEqual({
			settingId: 'mail.smtp.enabled',
			value: { case: 'booleanValue', value: true },
		})
	})

	it('round-trips CardDAV authority without credential or custom endpoint data', () => {
		const settings = create(ExportEffectiveOwnerModuleSettingsReceiptV1Schema, {
			registrationId: 'mail-registration',
			schemaMajor: 2,
			schemaRevision: MAIL_SETTINGS_SCHEMA_REVISION_V2,
			effectiveRevision: 4n,
			values: [
				stringSetting('mail.address_book.provider', 'icloud_carddav'),
				stringSetting('mail.address_book.carddav_username', 'owner@example.test'),
				stringSetting('mail.address_book.carddav_host', 'contacts.icloud.com'),
				unsignedSetting('mail.address_book.carddav_port', 443n),
				stringSetting('mail.address_book.carddav_base_path', '/'),
				stringSetting('mail.connection_id', 'mail-account'),
				stringSetting('mail.inbound.kind', 'imap'),
				stringSetting('mail.imap.host', 'imap.example.test'),
				unsignedSetting('mail.imap.port', 993n),
				stringSetting('mail.imap.username', 'owner@example.test'),
				unsignedSetting('mail.sync.window', 100n),
				unsignedSetting('mail.sync.windows', 2n),
				booleanSetting('mail.smtp.enabled', false),
			],
		})
		const status = create(MailAccountStatusV1Schema, {
			connectionId: 'mail-account',
			settingsRevision: 4n,
			runtimeGeneration: 5n,
			readiness: MailAccountReadinessV1.MAIL_ACCOUNT_READINESS_READY,
			connectorProfile: MailConnectorProfileV1.MAIL_CONNECTOR_PROFILE_IMAP,
			syncReadiness: MailProviderPathReadinessV1.MAIL_PROVIDER_PATH_READINESS_READY,
			deliveryReadiness: MailProviderPathReadinessV1.MAIL_PROVIDER_PATH_READINESS_NOT_CONFIGURED,
		})

		const exported = buildMailAccountExportV1(settings, status, 10n)
		const desired = mailAccountExportSettingsInputs(parseMailAccountExportV1(
			serializeMailAccountExportV1(exported),
		))

		expect(exported.configuration?.addressBookProvider).toBe(
			MailAddressBookProviderV1.MAIL_ADDRESS_BOOK_PROVIDER_ICLOUD_CARD_DAV,
		)
		expect(exported.configuration?.carddavUsername).toBe('owner@example.test')
		expect(desired).toContainEqual({
			settingId: 'mail.address_book.carddav_host',
			value: { case: 'stringValue', value: 'contacts.icloud.com' },
		})
		expect(serializeMailAccountExportV1(exported)).not.toMatch(/password|credential|secret/i)
	})

	it('rejects unknown secret-looking fields instead of silently dropping them', () => {
		const source = serializeMailAccountExportV1(buildMailAccountExportV1(
			create(ExportEffectiveOwnerModuleSettingsReceiptV1Schema, {
				registrationId: 'mail-registration',
				schemaMajor: 2,
					schemaRevision: MAIL_SETTINGS_SCHEMA_REVISION_V2,
				effectiveRevision: 1n,
				values: [
					stringSetting('mail.address_book.provider', 'none'),
					stringSetting('mail.connection_id', 'mail-account'),
					stringSetting('mail.inbound.kind', 'imap'),
					stringSetting('mail.imap.host', 'imap.example.test'),
					unsignedSetting('mail.imap.port', 993n),
					stringSetting('mail.imap.username', 'owner@example.test'),
					unsignedSetting('mail.sync.window', 100n),
					unsignedSetting('mail.sync.windows', 2n),
					booleanSetting('mail.smtp.enabled', false),
				],
			}),
			create(MailAccountStatusV1Schema, {
				connectionId: 'mail-account',
				settingsRevision: 1n,
				runtimeGeneration: 1n,
				readiness: MailAccountReadinessV1.MAIL_ACCOUNT_READINESS_CONFIGURATION_ONLY,
				connectorProfile: MailConnectorProfileV1.MAIL_CONNECTOR_PROFILE_IMAP,
				syncReadiness: MailProviderPathReadinessV1.MAIL_PROVIDER_PATH_READINESS_CREDENTIAL_REQUIRED,
				deliveryReadiness: MailProviderPathReadinessV1.MAIL_PROVIDER_PATH_READINESS_NOT_CONFIGURED,
			}),
		))
		const unsafe = JSON.parse(source) as Record<string, unknown>
		unsafe.password = 'must-not-be-ignored'

		expect(() => parseMailAccountExportV1(JSON.stringify(unsafe)))
			.toThrow('Mail account export is invalid')
	})
})

function stringSetting(settingId: string, value: string) {
	return create(OwnerSettingEntryV1Schema, {
		settingId,
		value: create(OwnerSettingValueV1Schema, {
			value: { case: 'stringValue', value },
		}),
	})
}

function unsignedSetting(settingId: string, value: bigint) {
	return create(OwnerSettingEntryV1Schema, {
		settingId,
		value: create(OwnerSettingValueV1Schema, {
			value: { case: 'unsignedIntegerValue', value },
		}),
	})
}

function booleanSetting(settingId: string, value: boolean) {
	return create(OwnerSettingEntryV1Schema, {
		settingId,
		value: create(OwnerSettingValueV1Schema, {
			value: { case: 'booleanValue', value },
		}),
	})
}
