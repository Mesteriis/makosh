import type { MailAccountConfigurationV1 } from '../../../gen/makosh/mail/portability/v1/portability_pb'
import { MailAddressBookProviderV1 } from '../../../gen/makosh/mail/portability/v1/portability_pb'
import type { OwnerSettingInputV1 } from '../../../platform/settings'

interface MailSettingsValueReaderPortV1 {
	string(settingId: string): string
	optionalString(settingId: string): string | undefined
	u32(settingId: string): number
}

const settingIds = {
	provider: 'mail.address_book.provider',
	cardDavUsername: 'mail.address_book.carddav_username',
	cardDavHost: 'mail.address_book.carddav_host',
	cardDavPort: 'mail.address_book.carddav_port',
	cardDavBasePath: 'mail.address_book.carddav_base_path',
	cardDavCa: 'mail.address_book.carddav_ca_certificate_pem',
	googlePeopleHost: 'mail.address_book.google_people_host',
	googlePeoplePort: 'mail.address_book.google_people_port',
	googlePeopleCa: 'mail.address_book.google_people_ca_certificate_pem',
} as const

const googlePeopleEndpoint = { host: 'people.googleapis.com', port: 443 } as const
const iCloudCardDavEndpoint = { host: 'contacts.icloud.com', port: 443, basePath: '/' } as const

export function readMailAddressBookPortabilityV1(
	values: MailSettingsValueReaderPortV1,
	inboundKind: string,
): { provider: MailAddressBookProviderV1, carddavUsername?: string } {
	const provider = values.string(settingIds.provider)
	if (provider === 'none') {
		return { provider: MailAddressBookProviderV1.MAIL_ADDRESS_BOOK_PROVIDER_NONE }
	}
	if (provider === 'google_people'
		&& inboundKind === 'gmail'
		&& values.string(settingIds.googlePeopleHost) === googlePeopleEndpoint.host
		&& values.u32(settingIds.googlePeoplePort) === googlePeopleEndpoint.port
		&& values.optionalString(settingIds.googlePeopleCa) === undefined) {
		return { provider: MailAddressBookProviderV1.MAIL_ADDRESS_BOOK_PROVIDER_GOOGLE_PEOPLE }
	}
	if (provider === 'icloud_carddav'
		&& inboundKind === 'imap'
		&& values.string(settingIds.cardDavHost) === iCloudCardDavEndpoint.host
		&& values.u32(settingIds.cardDavPort) === iCloudCardDavEndpoint.port
		&& values.string(settingIds.cardDavBasePath) === iCloudCardDavEndpoint.basePath
		&& values.optionalString(settingIds.cardDavCa) === undefined) {
		const carddavUsername = values.string(settingIds.cardDavUsername)
		if (boundedString(carddavUsername, 256)) {
			return {
				provider: MailAddressBookProviderV1.MAIL_ADDRESS_BOOK_PROVIDER_ICLOUD_CARD_DAV,
				carddavUsername,
			}
		}
	}
	throw invalidExport()
}

export function mailAddressBookSettingsInputsV1(
	configuration: MailAccountConfigurationV1,
): OwnerSettingInputV1[] {
	const inputs = [stringInput(
		settingIds.provider,
		exportAddressBookProviderV1(configuration.addressBookProvider),
	)]
	if (configuration.addressBookProvider
		=== MailAddressBookProviderV1.MAIL_ADDRESS_BOOK_PROVIDER_GOOGLE_PEOPLE) {
		inputs.push(
			stringInput(settingIds.googlePeopleHost, googlePeopleEndpoint.host),
			unsignedInput(settingIds.googlePeoplePort, googlePeopleEndpoint.port),
		)
	} else if (configuration.addressBookProvider
		=== MailAddressBookProviderV1.MAIL_ADDRESS_BOOK_PROVIDER_ICLOUD_CARD_DAV) {
		inputs.push(
			stringInput(settingIds.cardDavUsername, configuration.carddavUsername!),
			stringInput(settingIds.cardDavHost, iCloudCardDavEndpoint.host),
			unsignedInput(settingIds.cardDavPort, iCloudCardDavEndpoint.port),
			stringInput(settingIds.cardDavBasePath, iCloudCardDavEndpoint.basePath),
		)
	}
	return inputs
}

export function validMailAddressBookPortabilityV1(
	configuration: MailAccountConfigurationV1,
): boolean {
	switch (configuration.addressBookProvider) {
		case MailAddressBookProviderV1.MAIL_ADDRESS_BOOK_PROVIDER_NONE:
			return configuration.carddavUsername === undefined
		case MailAddressBookProviderV1.MAIL_ADDRESS_BOOK_PROVIDER_GOOGLE_PEOPLE:
			return configuration.inbound.case === 'gmail'
				&& configuration.carddavUsername === undefined
		case MailAddressBookProviderV1.MAIL_ADDRESS_BOOK_PROVIDER_ICLOUD_CARD_DAV:
			return configuration.inbound.case === 'imap'
				&& boundedString(configuration.carddavUsername ?? '', 256)
		default:
			return false
	}
}

function exportAddressBookProviderV1(provider: MailAddressBookProviderV1): string {
	switch (provider) {
		case MailAddressBookProviderV1.MAIL_ADDRESS_BOOK_PROVIDER_NONE:
			return 'none'
		case MailAddressBookProviderV1.MAIL_ADDRESS_BOOK_PROVIDER_GOOGLE_PEOPLE:
			return 'google_people'
		case MailAddressBookProviderV1.MAIL_ADDRESS_BOOK_PROVIDER_ICLOUD_CARD_DAV:
			return 'icloud_carddav'
		default:
			throw invalidExport()
	}
}

function stringInput(settingId: string, value: string): OwnerSettingInputV1 {
	return { settingId, value: { case: 'stringValue', value } }
}

function unsignedInput(settingId: string, value: number): OwnerSettingInputV1 {
	return { settingId, value: { case: 'unsignedIntegerValue', value: BigInt(value) } }
}

function boundedString(value: string, maximum: number): boolean {
	return value.trim().length > 0 && value.length <= maximum
}

function invalidExport(): Error {
	return new Error('Mail account export is invalid')
}
