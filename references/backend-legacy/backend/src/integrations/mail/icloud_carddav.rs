use std::time::Duration;

use makosh_communications_api::address_book::{AddressBookProviderBatch, AddressBookProviderEntry};
use quick_xml::escape::unescape;
use reqwest::{Client, Method};
use serde_json::Value;
use thiserror::Error;
use url::Url;

use crate::platform::secrets::models::ResolvedSecret;

const DEFAULT_ICLOUD_CARDDAV_URL: &str = "https://contacts.icloud.com/";
const XML_CONTENT_TYPE: &str = "application/xml; charset=utf-8";

#[derive(Debug, Error)]
pub enum IcloudCardDavError {
    #[error("iCloud CardDAV account configuration is incomplete")]
    InvalidConfig,
    #[error("iCloud CardDAV request failed: {0}")]
    Request(#[from] reqwest::Error),
    #[error("iCloud CardDAV returned HTTP {0}")]
    Http(reqwest::StatusCode),
    #[error("iCloud CardDAV discovery did not return an address book")]
    Discovery,
    #[error("iCloud CardDAV returned an invalid URL")]
    Url,
}

pub struct IcloudCardDavClient {
    client: Client,
    base_url: Url,
    username: String,
    password: String,
}

impl IcloudCardDavClient {
    pub fn from_config(
        config: &Value,
        password: &ResolvedSecret,
    ) -> Result<Self, IcloudCardDavError> {
        let username = config
            .get("username")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .ok_or(IcloudCardDavError::InvalidConfig)?;
        let base_url = config
            .get("carddav_base_url")
            .and_then(Value::as_str)
            .unwrap_or(DEFAULT_ICLOUD_CARDDAV_URL);
        Ok(Self {
            client: Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .map_err(IcloudCardDavError::Request)?,
            base_url: Url::parse(base_url).map_err(|_| IcloudCardDavError::Url)?,
            username: username.to_owned(),
            password: password.expose_for_runtime().to_owned(),
        })
    }

    pub async fn fetch_entries(&self) -> Result<AddressBookProviderBatch, IcloudCardDavError> {
        let discovery = self
            .propfind(self.base_url.clone(), discovery_body())
            .await?;
        let home_href = match first_property_href(&discovery, "addressbook-home-set") {
            Some(home_href) => home_href,
            None => {
                let principal_href = first_property_href(&discovery, "current-user-principal")
                    .ok_or(IcloudCardDavError::Discovery)?;
                let principal = self
                    .propfind(self.resolve_href(&principal_href)?, discovery_body())
                    .await?;
                first_property_href(&principal, "addressbook-home-set")
                    .ok_or(IcloudCardDavError::Discovery)?
            }
        };
        let home_url = self.resolve_href(&home_href)?;
        let home = self.propfind(home_url, allprop_body()).await?;
        let address_book_href = response_blocks(&home)
            .into_iter()
            .find(|block| block.contains("addressbook"))
            .and_then(|block| first_tag_text(block, "href"))
            .ok_or(IcloudCardDavError::Discovery)?;
        let address_book_url = self.resolve_href(&address_book_href)?;
        let response = self
            .report(address_book_url, addressbook_query_body())
            .await?;
        Ok(AddressBookProviderBatch {
            entries: response_blocks(&response)
                .into_iter()
                .filter_map(carddav_entry)
                .collect(),
            next_page_token: None,
        })
    }

    async fn propfind(&self, url: Url, body: &'static str) -> Result<String, IcloudCardDavError> {
        self.xml_request(
            Method::from_bytes(b"PROPFIND").expect("valid method"),
            url,
            "1",
            body,
        )
        .await
    }

    async fn report(&self, url: Url, body: &'static str) -> Result<String, IcloudCardDavError> {
        self.xml_request(
            Method::from_bytes(b"REPORT").expect("valid method"),
            url,
            "1",
            body,
        )
        .await
    }

    async fn xml_request(
        &self,
        method: Method,
        url: Url,
        depth: &'static str,
        body: &'static str,
    ) -> Result<String, IcloudCardDavError> {
        let response = self
            .client
            .request(method, url)
            .basic_auth(&self.username, Some(&self.password))
            .header("Depth", depth)
            .header("Content-Type", XML_CONTENT_TYPE)
            .body(body)
            .send()
            .await?;
        if !response.status().is_success() && response.status().as_u16() != 207 {
            return Err(IcloudCardDavError::Http(response.status()));
        }
        response.text().await.map_err(IcloudCardDavError::Request)
    }

    fn resolve_href(&self, href: &str) -> Result<Url, IcloudCardDavError> {
        self.base_url
            .join(href.trim())
            .map_err(|_| IcloudCardDavError::Url)
    }
}

fn carddav_entry(response: &str) -> Option<AddressBookProviderEntry> {
    let href = first_tag_text(response, "href")?;
    let card = first_tag_text(response, "address-data")?;
    let vcard = unescape(&card).ok()?.into_owned();
    let display_name = vcard_property(&vcard, "FN");
    let email_addresses = vcard_properties(&vcard, "EMAIL");
    let phone_numbers = vcard_properties(&vcard, "TEL");
    if display_name.is_none() && email_addresses.is_empty() && phone_numbers.is_empty() {
        return None;
    }
    Some(AddressBookProviderEntry {
        provider_address_book_entry_id: href,
        display_name,
        email_addresses,
        phone_numbers,
        etag: first_tag_text(response, "getetag"),
    })
}

fn response_blocks(xml: &str) -> Vec<&str> {
    let mut blocks = Vec::new();
    let mut remainder = xml;
    while let Some(start) = find_named_open_tag(remainder, "response") {
        let Some((tag, content_start)) = open_tag_name_and_content_start(&remainder[start..])
        else {
            break;
        };
        let close = format!("</{tag}>");
        let Some(end) = remainder[start + content_start..].find(&close) else {
            break;
        };
        let end = start + content_start + end + close.len();
        blocks.push(&remainder[start..end]);
        remainder = &remainder[end..];
    }
    blocks
}

fn first_property_href(xml: &str, property: &str) -> Option<String> {
    let start = find_named_open_tag(xml, property)?;
    let (_, content_start) = open_tag_name_and_content_start(&xml[start..])?;
    first_tag_text(&xml[start + content_start..], "href")
}

fn first_tag_text(xml: &str, name: &str) -> Option<String> {
    let start = find_named_open_tag(xml, name)?;
    let (tag, content_start) = open_tag_name_and_content_start(&xml[start..])?;
    let content_start = start + content_start;
    let close = format!("</{tag}>");
    let content_end = xml[content_start..].find(&close)? + content_start;
    Some(xml[content_start..content_end].trim().to_owned())
}

fn find_named_open_tag(xml: &str, name: &str) -> Option<usize> {
    let mut offset = 0;
    while let Some(found) = xml[offset..].find('<') {
        let start = offset + found;
        let Some((tag, _)) = open_tag_name_and_content_start(&xml[start..]) else {
            offset = start + 1;
            continue;
        };
        if tag == name || tag.ends_with(&format!(":{name}")) {
            return Some(start);
        }
        offset = start + 1;
    }
    None
}

fn open_tag_name_and_content_start(xml: &str) -> Option<(String, usize)> {
    let end = xml.find('>')?;
    let raw = xml.get(1..end)?.trim_start();
    if raw.starts_with('/') || raw.starts_with('?') || raw.starts_with('!') {
        return None;
    }
    let tag = raw
        .split_whitespace()
        .next()?
        .trim_end_matches('/')
        .to_owned();
    Some((tag, end + 1))
}

fn vcard_property(vcard: &str, name: &str) -> Option<String> {
    vcard_properties(vcard, name).into_iter().next()
}
fn vcard_properties(vcard: &str, name: &str) -> Vec<String> {
    unfold_vcard_lines(vcard)
        .lines()
        .filter_map(|line| {
            let (key, value) = line.split_once(':')?;
            (key.split(';').next()?.eq_ignore_ascii_case(name))
                .then(|| unescape_vcard_text(value.trim()))
                .filter(|value| !value.is_empty())
        })
        .collect()
}

fn unfold_vcard_lines(vcard: &str) -> String {
    let mut unfolded = String::with_capacity(vcard.len());
    for line in vcard.replace("\r\n", "\n").split('\n') {
        if line.starts_with(' ') || line.starts_with('\t') {
            unfolded.push_str(line.trim_start_matches([' ', '\t']));
        } else {
            if !unfolded.is_empty() {
                unfolded.push('\n');
            }
            unfolded.push_str(line);
        }
    }
    unfolded
}

fn unescape_vcard_text(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
    let mut chars = value.chars();
    while let Some(ch) = chars.next() {
        if ch != '\\' {
            output.push(ch);
            continue;
        }
        match chars.next() {
            Some('n' | 'N') => output.push('\n'),
            Some(',') => output.push(','),
            Some(';') => output.push(';'),
            Some('\\') => output.push('\\'),
            Some(other) => {
                output.push('\\');
                output.push(other);
            }
            None => output.push('\\'),
        }
    }
    output
}

fn discovery_body() -> &'static str {
    "<?xml version=\"1.0\"?><propfind xmlns=\"DAV:\"><prop><current-user-principal/><addressbook-home-set xmlns=\"urn:ietf:params:xml:ns:carddav\"/></prop></propfind>"
}
fn allprop_body() -> &'static str {
    "<?xml version=\"1.0\"?><propfind xmlns=\"DAV:\"><allprop/></propfind>"
}
fn addressbook_query_body() -> &'static str {
    "<?xml version=\"1.0\"?><card:addressbook-query xmlns:d=\"DAV:\" xmlns:card=\"urn:ietf:params:xml:ns:carddav\"><d:prop><d:getetag/><d:href/><card:address-data/></d:prop></card:addressbook-query>"
}

#[cfg(test)]
mod tests {
    use super::carddav_entry;

    #[test]
    fn parses_vcard_contact_from_carddav_response() {
        let response = "<response><href>/card/ada.vcf</href><getetag>\"v1\"</getetag><address-data>BEGIN:VCARD\nFN:Ada Lovelace\nEMAIL;TYPE=INTERNET:ada@example.test\nTEL;TYPE=CELL:+123\nEND:VCARD</address-data></response>";
        let entry = carddav_entry(response).expect("CardDAV entry");
        assert_eq!(entry.provider_address_book_entry_id, "/card/ada.vcf");
        assert_eq!(entry.display_name.as_deref(), Some("Ada Lovelace"));
        assert_eq!(entry.email_addresses, vec!["ada@example.test"]);
        assert_eq!(entry.phone_numbers, vec!["+123"]);
    }

    #[test]
    fn parses_namespaced_folded_and_escaped_vcard_contact() {
        let response = r#"<d:response xmlns:d="DAV:" xmlns:card="urn:ietf:params:xml:ns:carddav"><d:href>/card/grace.vcf</d:href><d:getetag>"v2"</d:getetag><card:address-data>BEGIN:VCARD&#10;FN:Grace\, Hopper&#10;EMAIL;TYPE=INTERNET:grace@exam&#10; ple.test&#10;TEL;TYPE=CELL:+456&#10;END:VCARD</card:address-data></d:response>"#;
        let entry = carddav_entry(response).expect("namespaced CardDAV entry");

        assert_eq!(entry.provider_address_book_entry_id, "/card/grace.vcf");
        assert_eq!(entry.display_name.as_deref(), Some("Grace, Hopper"));
        assert_eq!(entry.email_addresses, vec!["grace@example.test"]);
        assert_eq!(entry.phone_numbers, vec!["+456"]);
        assert_eq!(entry.etag.as_deref(), Some("\"v2\""));
    }
}
