use std::net::SocketAddr;
use std::time::Duration;

use axum::http::{HeaderValue, header};
use reqwest::Url;

use super::dns::resolve_public_image_addrs;
use super::errors::RemoteImageFetchError;

const MAX_REMOTE_IMAGE_BYTES: u64 = 12 * 1024 * 1024;
const REMOTE_IMAGE_TIMEOUT: Duration = Duration::from_secs(15);

pub(super) struct RemoteImage {
    pub(super) content_type: HeaderValue,
    pub(super) body: Vec<u8>,
}

pub(super) async fn fetch_remote_image(url: &Url) -> Result<RemoteImage, RemoteImageFetchError> {
    let default_client = remote_image_client(None)?;
    match fetch_remote_image_with_client(&default_client, url).await {
        Ok(image) => Ok(image),
        Err(first_error @ RemoteImageFetchError::Http(_)) => {
            let Some(host) = url.host_str() else {
                return Err(RemoteImageFetchError::MissingHost);
            };
            let port = url.port_or_known_default().unwrap_or(443);
            let public_addrs = resolve_public_image_addrs(host, port).await?;
            if public_addrs.is_empty() {
                return Err(first_error);
            }
            let fallback_client = remote_image_client(Some((host, public_addrs.as_slice())))?;
            fetch_remote_image_with_client(&fallback_client, url).await
        }
        Err(error) => Err(error),
    }
}

fn remote_image_client(
    dns_override: Option<(&str, &[SocketAddr])>,
) -> Result<reqwest::Client, RemoteImageFetchError> {
    let mut builder = reqwest::Client::builder()
        .timeout(REMOTE_IMAGE_TIMEOUT)
        .redirect(reqwest::redirect::Policy::limited(4))
        .user_agent("МакошьHub-MailImageProxy/0.1");
    if let Some((host, addrs)) = dns_override {
        builder = builder.resolve_to_addrs(host, addrs);
    }
    Ok(builder.build()?)
}

async fn fetch_remote_image_with_client(
    client: &reqwest::Client,
    url: &Url,
) -> Result<RemoteImage, RemoteImageFetchError> {
    let response = client.get(url.clone()).send().await?;
    if !response.status().is_success() {
        return Err(RemoteImageFetchError::NonSuccessStatus);
    }
    if response.content_length().unwrap_or(0) > MAX_REMOTE_IMAGE_BYTES {
        return Err(RemoteImageFetchError::TooLarge);
    }
    let content_type = response
        .headers()
        .get(header::CONTENT_TYPE)
        .cloned()
        .ok_or(RemoteImageFetchError::InvalidHeader)?;
    let content_type_text = content_type
        .to_str()
        .map_err(|_| RemoteImageFetchError::InvalidHeader)?
        .to_ascii_lowercase();
    if !content_type_text.starts_with("image/") {
        return Err(RemoteImageFetchError::NotImage);
    }
    let body = response.bytes().await?;
    if body.len() as u64 > MAX_REMOTE_IMAGE_BYTES {
        return Err(RemoteImageFetchError::TooLarge);
    }
    Ok(RemoteImage {
        content_type,
        body: body.to_vec(),
    })
}
