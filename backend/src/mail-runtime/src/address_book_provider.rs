//! Mail-owned construction of address-book provider clients from admitted settings.

use makosh_mail_api::{MailAddressBookTlsEndpointV1, MailCardDavEndpointV1};
use makosh_mail_carddav::{CardDavAdapterErrorV1, CardDavClientV1};
use makosh_mail_google_people::{GooglePeopleAdapterErrorV1, GooglePeopleClientV1};

pub(crate) fn google_people_client_v1(
    endpoint: &MailAddressBookTlsEndpointV1,
) -> Result<GooglePeopleClientV1, GooglePeopleAdapterErrorV1> {
    #[cfg(feature = "conformance-test-support")]
    {
        GooglePeopleClientV1::for_conformance_endpoint(
            &endpoint.host,
            endpoint.port,
            endpoint.ca_certificate_pem.clone(),
        )
    }
    #[cfg(not(feature = "conformance-test-support"))]
    {
        if endpoint.host != makosh_mail_api::GOOGLE_PEOPLE_API_HOST_V1
            || endpoint.port != makosh_mail_api::GOOGLE_PEOPLE_API_PORT_V1
            || endpoint.ca_certificate_pem.is_some()
        {
            return Err(GooglePeopleAdapterErrorV1::InvalidRequest);
        }
        GooglePeopleClientV1::new()
    }
}

pub(crate) fn carddav_client_v1(
    endpoint: &MailCardDavEndpointV1,
) -> Result<CardDavClientV1, CardDavAdapterErrorV1> {
    #[cfg(feature = "conformance-test-support")]
    {
        CardDavClientV1::for_conformance_endpoint(
            &endpoint.tls.host,
            endpoint.tls.port,
            &endpoint.base_path,
            endpoint.tls.ca_certificate_pem.clone(),
        )
    }
    #[cfg(not(feature = "conformance-test-support"))]
    {
        if endpoint.tls.host != makosh_mail_api::ICLOUD_CARDDAV_HOST_V1
            || endpoint.tls.port != makosh_mail_api::ICLOUD_CARDDAV_PORT_V1
            || endpoint.base_path != makosh_mail_api::ICLOUD_CARDDAV_BASE_PATH_V1
            || endpoint.tls.ca_certificate_pem.is_some()
        {
            return Err(CardDavAdapterErrorV1::InvalidRequest);
        }
        CardDavClientV1::new()
    }
}
