import {
	create,
	fromJsonString,
	toJsonString,
} from '@bufbuild/protobuf'

import {
	MailAccountReadinessV1,
	type MailAccountStatusV1,
	MailConnectorProfileV1,
	MailProviderPathReadinessV1,
} from '../../../gen/makosh/mail/account/v1/client_pb'
import {
	MailAccountConfigurationV1Schema,
	type MailAccountExportV1,
	MailAccountExportV1Schema,
	MailExportAccountReadinessV1,
	MailExportConnectorProfileV1,
	MailExportProviderPathReadinessV1,
	MailGmailConfigurationV1Schema,
	MailHttpEndpointV1Schema,
	MailImapConfigurationV1Schema,
	MailSmtpConfigurationV1Schema,
	MailTlsEndpointV1Schema,
} from '../../../gen/makosh/mail/portability/v1/portability_pb'
import type {
	ExportEffectiveOwnerModuleSettingsReceiptV1,
	OwnerSettingEntryV1,
} from '../../../gen/makosh/gateway/v1/owner_module_settings_pb'
import type { OwnerSettingInputV1 } from '../../../platform/settings'
import {
	mailAddressBookSettingsInputsV1,
	readMailAddressBookPortabilityV1,
	validMailAddressBookPortabilityV1,
} from './mailAddressBookPortabilityCodec'

export const MAIL_ACCOUNT_EXPORT_MAJOR_V1 = 1
export const MAIL_SETTINGS_SCHEMA_MAJOR_V2 = 2
export const MAIL_SETTINGS_SCHEMA_REVISION_V2 = 4

const settingIds = {
	connectionId: 'mail.connection_id',
	imapHost: 'mail.imap.host',
	imapPort: 'mail.imap.port',
	imapUsername: 'mail.imap.username',
	syncWindow: 'mail.sync.window',
	syncWindows: 'mail.sync.windows',
	smtpEnabled: 'mail.smtp.enabled',
	smtpCa: 'mail.smtp.ca_certificate_pem',
	smtpHost: 'mail.smtp.host',
	smtpPort: 'mail.smtp.port',
	smtpUsername: 'mail.smtp.username',
	smtpFrom: 'mail.smtp.from_address',
	inboundKind: 'mail.inbound.kind',
	gmailApiHost: 'mail.gmail.api_host',
	gmailApiPort: 'mail.gmail.api_port',
	gmailCa: 'mail.gmail.ca_certificate_pem',
	gmailUserId: 'mail.gmail.user_id',
	gmailFrom: 'mail.gmail.from_address',
	gmailOauthAuthorizationCa: 'mail.gmail.oauth.authorization_ca_certificate_pem',
	gmailOauthAuthorizationHost: 'mail.gmail.oauth.authorization_host',
	gmailOauthAuthorizationPath: 'mail.gmail.oauth.authorization_path',
	gmailOauthAuthorizationPort: 'mail.gmail.oauth.authorization_port',
	gmailOauthClientId: 'mail.gmail.oauth.client_id',
	gmailOauthRedirectUri: 'mail.gmail.oauth.redirect_uri',
	gmailOauthTokenCa: 'mail.gmail.oauth.token_ca_certificate_pem',
	gmailOauthTokenHost: 'mail.gmail.oauth.token_host',
	gmailOauthTokenPath: 'mail.gmail.oauth.token_path',
	gmailOauthTokenPort: 'mail.gmail.oauth.token_port',
} as const

export function buildMailAccountExportV1(
	settings: ExportEffectiveOwnerModuleSettingsReceiptV1,
	status: MailAccountStatusV1,
	exportedAtUnixMillis = BigInt(Date.now()),
): MailAccountExportV1 {
	if (settings.schemaMajor !== MAIL_SETTINGS_SCHEMA_MAJOR_V2
		|| settings.schemaRevision !== MAIL_SETTINGS_SCHEMA_REVISION_V2
		|| settings.effectiveRevision !== status.settingsRevision) {
		throw invalidExport()
	}
	const values = new MailSettingsValueReaderV1(settings.values)
	const connectionId = values.string(settingIds.connectionId)
	if (connectionId !== status.connectionId) throw invalidExport()
	const inboundKind = values.string(settingIds.inboundKind)
	const smtpEnabled = values.boolean(settingIds.smtpEnabled)
	const addressBook = readMailAddressBookPortabilityV1(values, inboundKind)
	const configuration = create(MailAccountConfigurationV1Schema, {
		connectionId,
		syncWindow: values.u32(settingIds.syncWindow),
		syncWindows: values.u32(settingIds.syncWindows),
		inbound: inboundKind === 'imap'
			? {
				case: 'imap',
				value: create(MailImapConfigurationV1Schema, {
					host: values.string(settingIds.imapHost),
					port: values.u32(settingIds.imapPort),
					username: values.string(settingIds.imapUsername),
				}),
			}
			: inboundKind === 'gmail'
				? {
					case: 'gmail',
					value: create(MailGmailConfigurationV1Schema, {
						userId: values.string(settingIds.gmailUserId),
						fromAddress: values.string(settingIds.gmailFrom),
						apiEndpoint: create(MailTlsEndpointV1Schema, {
							host: values.string(settingIds.gmailApiHost),
							port: values.u32(settingIds.gmailApiPort),
							caCertificatePem: values.optionalString(settingIds.gmailCa),
						}),
						oauthClientId: values.string(settingIds.gmailOauthClientId),
						oauthRedirectUri: values.string(settingIds.gmailOauthRedirectUri),
						oauthAuthorizationEndpoint: create(MailHttpEndpointV1Schema, {
							host: values.string(settingIds.gmailOauthAuthorizationHost),
							port: values.u32(settingIds.gmailOauthAuthorizationPort),
							path: values.string(settingIds.gmailOauthAuthorizationPath),
							caCertificatePem: values.optionalString(
								settingIds.gmailOauthAuthorizationCa,
							),
						}),
						oauthTokenEndpoint: create(MailHttpEndpointV1Schema, {
							host: values.string(settingIds.gmailOauthTokenHost),
							port: values.u32(settingIds.gmailOauthTokenPort),
							path: values.string(settingIds.gmailOauthTokenPath),
							caCertificatePem: values.optionalString(settingIds.gmailOauthTokenCa),
						}),
					}),
				}
				: { case: undefined },
		smtp: smtpEnabled
			? create(MailSmtpConfigurationV1Schema, {
				host: values.string(settingIds.smtpHost),
				port: values.u32(settingIds.smtpPort),
				username: values.string(settingIds.smtpUsername),
				fromAddress: values.string(settingIds.smtpFrom),
				caCertificatePem: values.optionalString(settingIds.smtpCa),
			})
			: undefined,
		addressBookProvider: addressBook.provider,
		carddavUsername: addressBook.carddavUsername,
	})
	const exported = create(MailAccountExportV1Schema, {
		major: MAIL_ACCOUNT_EXPORT_MAJOR_V1,
		exportedAtUnixMillis,
		sourceRegistrationId: settings.registrationId,
		settingsSchemaMajor: settings.schemaMajor,
		settingsSchemaRevision: settings.schemaRevision,
		effectiveSettingsRevision: settings.effectiveRevision,
		connectorProfile: exportConnectorProfile(status.connectorProfile),
		readiness: exportAccountReadiness(status.readiness),
		syncReadiness: exportPathReadiness(status.syncReadiness),
		deliveryReadiness: exportPathReadiness(status.deliveryReadiness),
		configuration,
	})
	validateMailAccountExportV1(exported)
	return exported
}

export function serializeMailAccountExportV1(exported: MailAccountExportV1): string {
	validateMailAccountExportV1(exported)
	return toJsonString(MailAccountExportV1Schema, exported, { prettySpaces: 2 })
}

export function parseMailAccountExportV1(source: string): MailAccountExportV1 {
	let exported: MailAccountExportV1
	try {
		exported = fromJsonString(MailAccountExportV1Schema, source, {
			ignoreUnknownFields: false,
		})
	} catch {
		throw invalidExport()
	}
	validateMailAccountExportV1(exported)
	return exported
}

export function mailAccountExportSettingsInputs(
	exported: MailAccountExportV1,
): OwnerSettingInputV1[] {
	validateMailAccountExportV1(exported)
	const configuration = exported.configuration!
	const inputs: OwnerSettingInputV1[] = [
		...mailAddressBookSettingsInputsV1(configuration),
		stringInput(settingIds.connectionId, configuration.connectionId),
		unsignedInput(settingIds.syncWindow, configuration.syncWindow),
		unsignedInput(settingIds.syncWindows, configuration.syncWindows),
		booleanInput(settingIds.smtpEnabled, configuration.smtp !== undefined),
	]
	if (configuration.inbound.case === 'imap') {
		inputs.push(
			stringInput(settingIds.inboundKind, 'imap'),
			stringInput(settingIds.imapHost, configuration.inbound.value.host),
			unsignedInput(settingIds.imapPort, configuration.inbound.value.port),
			stringInput(settingIds.imapUsername, configuration.inbound.value.username),
		)
	} else if (configuration.inbound.case === 'gmail') {
		const gmail = configuration.inbound.value
		inputs.push(
			stringInput(settingIds.inboundKind, 'gmail'),
			stringInput(settingIds.gmailApiHost, gmail.apiEndpoint!.host),
			unsignedInput(settingIds.gmailApiPort, gmail.apiEndpoint!.port),
			stringInput(settingIds.gmailUserId, gmail.userId),
			stringInput(settingIds.gmailFrom, gmail.fromAddress),
			stringInput(settingIds.gmailOauthClientId, gmail.oauthClientId),
			stringInput(settingIds.gmailOauthRedirectUri, gmail.oauthRedirectUri),
			stringInput(
				settingIds.gmailOauthAuthorizationHost,
				gmail.oauthAuthorizationEndpoint!.host,
			),
			unsignedInput(
				settingIds.gmailOauthAuthorizationPort,
				gmail.oauthAuthorizationEndpoint!.port,
			),
			stringInput(
				settingIds.gmailOauthAuthorizationPath,
				gmail.oauthAuthorizationEndpoint!.path,
			),
			stringInput(settingIds.gmailOauthTokenHost, gmail.oauthTokenEndpoint!.host),
			unsignedInput(settingIds.gmailOauthTokenPort, gmail.oauthTokenEndpoint!.port),
			stringInput(settingIds.gmailOauthTokenPath, gmail.oauthTokenEndpoint!.path),
		)
		optionalStringInput(inputs, settingIds.gmailCa, gmail.apiEndpoint!.caCertificatePem)
		optionalStringInput(
			inputs,
			settingIds.gmailOauthAuthorizationCa,
			gmail.oauthAuthorizationEndpoint!.caCertificatePem,
		)
		optionalStringInput(
			inputs,
			settingIds.gmailOauthTokenCa,
			gmail.oauthTokenEndpoint!.caCertificatePem,
		)
	}
	if (configuration.smtp) {
		inputs.push(
			stringInput(settingIds.smtpHost, configuration.smtp.host),
			unsignedInput(settingIds.smtpPort, configuration.smtp.port),
			stringInput(settingIds.smtpUsername, configuration.smtp.username),
			stringInput(settingIds.smtpFrom, configuration.smtp.fromAddress),
		)
		optionalStringInput(inputs, settingIds.smtpCa, configuration.smtp.caCertificatePem)
	}
	return inputs.sort((left, right) => left.settingId.localeCompare(right.settingId))
}

export function validateMailAccountExportV1(exported: MailAccountExportV1): void {
	const configuration = exported.configuration
	if (exported.major !== MAIL_ACCOUNT_EXPORT_MAJOR_V1
		|| exported.exportedAtUnixMillis <= 0n
		|| !boundedString(exported.sourceRegistrationId, 128)
		|| exported.settingsSchemaMajor !== MAIL_SETTINGS_SCHEMA_MAJOR_V2
		|| exported.settingsSchemaRevision !== MAIL_SETTINGS_SCHEMA_REVISION_V2
		|| exported.effectiveSettingsRevision <= 0n
		|| !configuration
		|| !boundedString(configuration.connectionId, 128)
		|| configuration.syncWindow <= 0
		|| configuration.syncWindows <= 0
		|| configuration.inbound.case === undefined
		|| !validMailAddressBookPortabilityV1(configuration)
		|| exported.connectorProfile === MailExportConnectorProfileV1.MAIL_EXPORT_CONNECTOR_PROFILE_UNSPECIFIED
		|| exported.readiness === MailExportAccountReadinessV1.MAIL_EXPORT_ACCOUNT_READINESS_UNSPECIFIED
		|| exported.syncReadiness === MailExportProviderPathReadinessV1.MAIL_EXPORT_PROVIDER_PATH_READINESS_UNSPECIFIED
		|| exported.deliveryReadiness === MailExportProviderPathReadinessV1.MAIL_EXPORT_PROVIDER_PATH_READINESS_UNSPECIFIED) {
		throw invalidExport()
	}
	if (configuration.inbound.case === 'imap') {
		const imap = configuration.inbound.value
		const expected = configuration.smtp
			? MailExportConnectorProfileV1.MAIL_EXPORT_CONNECTOR_PROFILE_IMAP_SMTP
			: MailExportConnectorProfileV1.MAIL_EXPORT_CONNECTOR_PROFILE_IMAP
		if (exported.connectorProfile !== expected
			|| !validEndpoint(imap.host, imap.port)
			|| !boundedString(imap.username, 512)
			|| (configuration.smtp !== undefined
				&& (!validEndpoint(configuration.smtp.host, configuration.smtp.port)
					|| !boundedString(configuration.smtp.username, 512)
					|| !boundedString(configuration.smtp.fromAddress, 320)))) {
			throw invalidExport()
		}
		return
	}
	const gmail = configuration.inbound.value
	if (exported.connectorProfile !== MailExportConnectorProfileV1.MAIL_EXPORT_CONNECTOR_PROFILE_GMAIL
		|| configuration.smtp !== undefined
		|| !boundedString(gmail.userId, 512)
		|| !boundedString(gmail.fromAddress, 320)
		|| !gmail.apiEndpoint
		|| !validEndpoint(gmail.apiEndpoint.host, gmail.apiEndpoint.port)
		|| !boundedString(gmail.oauthClientId, 4096)
		|| !boundedString(gmail.oauthRedirectUri, 4096)
		|| !gmail.oauthAuthorizationEndpoint
		|| !validHttpEndpoint(gmail.oauthAuthorizationEndpoint)
		|| !gmail.oauthTokenEndpoint
		|| !validHttpEndpoint(gmail.oauthTokenEndpoint)) {
		throw invalidExport()
	}
}

class MailSettingsValueReaderV1 {
	private readonly values = new Map<string, OwnerSettingEntryV1>()

	constructor(entries: OwnerSettingEntryV1[]) {
		for (const entry of entries) {
			if (this.values.has(entry.settingId) || !entry.value) throw invalidExport()
			this.values.set(entry.settingId, entry)
		}
	}

	string(settingId: string): string {
		const value = this.values.get(settingId)?.value?.value
		if (value?.case !== 'stringValue' || value.value.length === 0) throw invalidExport()
		return value.value
	}

	optionalString(settingId: string): string | undefined {
		const entry = this.values.get(settingId)
		if (!entry) return undefined
		const value = entry.value?.value
		if (value?.case !== 'stringValue') throw invalidExport()
		return value.value
	}

	boolean(settingId: string): boolean {
		const value = this.values.get(settingId)?.value?.value
		if (value?.case !== 'booleanValue') throw invalidExport()
		return value.value
	}

	u32(settingId: string): number {
		const value = this.values.get(settingId)?.value?.value
		if (value?.case !== 'unsignedIntegerValue'
			|| value.value <= 0n
			|| value.value > BigInt(0xffff_ffff)) {
			throw invalidExport()
		}
		return Number(value.value)
	}
}

function stringInput(settingId: string, value: string): OwnerSettingInputV1 {
	return { settingId, value: { case: 'stringValue', value } }
}

function unsignedInput(settingId: string, value: number): OwnerSettingInputV1 {
	return { settingId, value: { case: 'unsignedIntegerValue', value: BigInt(value) } }
}

function booleanInput(settingId: string, value: boolean): OwnerSettingInputV1 {
	return { settingId, value: { case: 'booleanValue', value } }
}

function optionalStringInput(
	inputs: OwnerSettingInputV1[],
	settingId: string,
	value: string | undefined,
): void {
	if (value !== undefined) inputs.push(stringInput(settingId, value))
}

function exportConnectorProfile(value: MailConnectorProfileV1): MailExportConnectorProfileV1 {
	switch (value) {
		case MailConnectorProfileV1.MAIL_CONNECTOR_PROFILE_IMAP:
			return MailExportConnectorProfileV1.MAIL_EXPORT_CONNECTOR_PROFILE_IMAP
		case MailConnectorProfileV1.MAIL_CONNECTOR_PROFILE_IMAP_SMTP:
			return MailExportConnectorProfileV1.MAIL_EXPORT_CONNECTOR_PROFILE_IMAP_SMTP
		case MailConnectorProfileV1.MAIL_CONNECTOR_PROFILE_GMAIL:
			return MailExportConnectorProfileV1.MAIL_EXPORT_CONNECTOR_PROFILE_GMAIL
		default:
			throw invalidExport()
	}
}

function exportAccountReadiness(value: MailAccountReadinessV1): MailExportAccountReadinessV1 {
	switch (value) {
		case MailAccountReadinessV1.MAIL_ACCOUNT_READINESS_CONFIGURATION_ONLY:
			return MailExportAccountReadinessV1.MAIL_EXPORT_ACCOUNT_READINESS_CONFIGURATION_ONLY
		case MailAccountReadinessV1.MAIL_ACCOUNT_READINESS_PENDING_RESTART:
			return MailExportAccountReadinessV1.MAIL_EXPORT_ACCOUNT_READINESS_PENDING_RESTART
		case MailAccountReadinessV1.MAIL_ACCOUNT_READINESS_READY:
			return MailExportAccountReadinessV1.MAIL_EXPORT_ACCOUNT_READINESS_READY
		case MailAccountReadinessV1.MAIL_ACCOUNT_READINESS_RETIRED:
			return MailExportAccountReadinessV1.MAIL_EXPORT_ACCOUNT_READINESS_RETIRED
		case MailAccountReadinessV1.MAIL_ACCOUNT_READINESS_DELETED:
			return MailExportAccountReadinessV1.MAIL_EXPORT_ACCOUNT_READINESS_DELETED
		case MailAccountReadinessV1.MAIL_ACCOUNT_READINESS_DEGRADED:
			return MailExportAccountReadinessV1.MAIL_EXPORT_ACCOUNT_READINESS_DEGRADED
		default:
			throw invalidExport()
	}
}

function exportPathReadiness(
	value: MailProviderPathReadinessV1,
): MailExportProviderPathReadinessV1 {
	switch (value) {
		case MailProviderPathReadinessV1.MAIL_PROVIDER_PATH_READINESS_NOT_CONFIGURED:
			return MailExportProviderPathReadinessV1.MAIL_EXPORT_PROVIDER_PATH_READINESS_NOT_CONFIGURED
		case MailProviderPathReadinessV1.MAIL_PROVIDER_PATH_READINESS_CREDENTIAL_REQUIRED:
			return MailExportProviderPathReadinessV1.MAIL_EXPORT_PROVIDER_PATH_READINESS_CREDENTIAL_REQUIRED
		case MailProviderPathReadinessV1.MAIL_PROVIDER_PATH_READINESS_PENDING_RESTART:
			return MailExportProviderPathReadinessV1.MAIL_EXPORT_PROVIDER_PATH_READINESS_PENDING_RESTART
		case MailProviderPathReadinessV1.MAIL_PROVIDER_PATH_READINESS_READY:
			return MailExportProviderPathReadinessV1.MAIL_EXPORT_PROVIDER_PATH_READINESS_READY
		case MailProviderPathReadinessV1.MAIL_PROVIDER_PATH_READINESS_RETIRED:
			return MailExportProviderPathReadinessV1.MAIL_EXPORT_PROVIDER_PATH_READINESS_RETIRED
		case MailProviderPathReadinessV1.MAIL_PROVIDER_PATH_READINESS_DELETED:
			return MailExportProviderPathReadinessV1.MAIL_EXPORT_PROVIDER_PATH_READINESS_DELETED
		case MailProviderPathReadinessV1.MAIL_PROVIDER_PATH_READINESS_DEGRADED:
			return MailExportProviderPathReadinessV1.MAIL_EXPORT_PROVIDER_PATH_READINESS_DEGRADED
		default:
			throw invalidExport()
	}
}

function validEndpoint(host: string, port: number): boolean {
	return boundedString(host, 253)
		&& Number.isInteger(port)
		&& port > 0
		&& port <= 65_535
}

function validHttpEndpoint(endpoint: {
	host: string
	port: number
	path: string
}): boolean {
	return validEndpoint(endpoint.host, endpoint.port)
		&& boundedString(endpoint.path, 2048)
		&& endpoint.path.startsWith('/')
}

function boundedString(value: string, maximum: number): boolean {
	return value.trim().length > 0 && value.length <= maximum
}

function invalidExport(): Error {
	return new Error('Mail account export is invalid')
}
