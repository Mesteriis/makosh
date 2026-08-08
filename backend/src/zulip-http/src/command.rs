use makosh_zulip_api::{ZulipCommandV1, ZulipReactionOperationV1, direct_recipient_user_ids};

use crate::{ZulipHttpConfigV1, wire::ZulipHttpErrorV1};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ZulipHttpRequestV1 {
    pub method: &'static str,
    pub path: String,
    pub form_body: String,
    pub content_type: &'static str,
    pub body: Vec<u8>,
}

pub fn request_for_command(
    config: &ZulipHttpConfigV1,
    command: &ZulipCommandV1,
) -> Result<ZulipHttpRequestV1, ZulipHttpErrorV1> {
    let account_id = command_account_id(command);
    if account_id != config.account.account_id {
        return Err(ZulipHttpErrorV1::InvalidCommand);
    }
    let base_path = realm_path(&config.account.realm_url)?;
    match command {
        ZulipCommandV1::SendStream {
            stream,
            topic,
            content,
            ..
        } => request(
            "POST",
            format!("{base_path}api/v1/messages"),
            &[
                ("type", "stream".to_owned()),
                ("to", required(stream)?),
                ("topic", required(topic)?),
                ("content", required(content)?),
            ],
        ),
        ZulipCommandV1::SendDirect {
            recipients,
            content,
            ..
        } => {
            let recipient_json = match direct_recipient_user_ids(recipients) {
                Some(ids) => format!(
                    "[{}]",
                    ids.into_iter()
                        .map(|value| value.to_string())
                        .collect::<Vec<_>>()
                        .join(",")
                ),
                None => json_string_array(recipients)?,
            };
            request(
                "POST",
                format!("{base_path}api/v1/messages"),
                &[
                    ("type", "direct".to_owned()),
                    ("to", recipient_json),
                    ("content", required(content)?),
                ],
            )
        }
        ZulipCommandV1::UpdateMessage {
            provider_message_id,
            content,
            topic,
            ..
        } => {
            let message_id = message_id(provider_message_id)?;
            let mut fields = Vec::new();
            if let Some(content) = content {
                fields.push(("content", required(content)?));
            }
            if let Some(topic) = topic {
                fields.push(("topic", required(topic)?));
            }
            (!fields.is_empty())
                .then_some(())
                .ok_or(ZulipHttpErrorV1::InvalidCommand)?;
            request(
                "PATCH",
                format!("{base_path}api/v1/messages/{message_id}"),
                &fields,
            )
        }
        ZulipCommandV1::DeleteMessage {
            provider_message_id,
            ..
        } => request(
            "DELETE",
            format!(
                "{base_path}api/v1/messages/{}",
                message_id(provider_message_id)?
            ),
            &[],
        ),
        ZulipCommandV1::Reaction {
            provider_message_id,
            reaction,
            operation,
            ..
        } => {
            let mut fields = vec![("emoji_name", required(&reaction.emoji_name)?)];
            if let Some(emoji_code) = &reaction.emoji_code {
                fields.push(("emoji_code", required(emoji_code)?));
            }
            if let Some(reaction_type) = &reaction.reaction_type {
                fields.push(("reaction_type", required(reaction_type)?));
            }
            request(
                match operation {
                    ZulipReactionOperationV1::Add => "POST",
                    ZulipReactionOperationV1::Remove => "DELETE",
                },
                format!(
                    "{base_path}api/v1/messages/{}/reactions",
                    message_id(provider_message_id)?
                ),
                &fields,
            )
        }
        ZulipCommandV1::SendStreamWithUpload { .. }
        | ZulipCommandV1::SendDirectWithUpload { .. }
        | ZulipCommandV1::DownloadAttachment { .. } => Err(ZulipHttpErrorV1::InvalidCommand),
    }
}

pub fn request_for_queue_registration(
    config: &ZulipHttpConfigV1,
) -> Result<ZulipHttpRequestV1, ZulipHttpErrorV1> {
    let base_path = realm_path(&config.account.realm_url)?;
    request(
        "POST",
        format!("{base_path}api/v1/register"),
        &[(
            "event_types",
            "[\"message\",\"update_message\",\"delete_message\",\"reaction\"]".to_owned(),
        )],
    )
}

pub fn request_for_queue_poll(
    config: &ZulipHttpConfigV1,
    queue_id: &str,
    last_event_id: i64,
) -> Result<ZulipHttpRequestV1, ZulipHttpErrorV1> {
    let queue_id = required(queue_id)?;
    (last_event_id >= 0)
        .then_some(())
        .ok_or(ZulipHttpErrorV1::InvalidCommand)?;
    let base_path = realm_path(&config.account.realm_url)?;
    Ok(ZulipHttpRequestV1 {
        method: "GET",
        path: format!(
            "{base_path}api/v1/events?queue_id={}&last_event_id={last_event_id}&dont_block=true",
            percent_encode(&queue_id)
        ),
        form_body: String::new(),
        content_type: "application/x-www-form-urlencoded",
        body: Vec::new(),
    })
}

pub fn request_for_message_history(
    config: &ZulipHttpConfigV1,
    before_provider_message_id: Option<&str>,
    limit: u32,
) -> Result<ZulipHttpRequestV1, ZulipHttpErrorV1> {
    if limit == 0 || limit > 200 {
        return Err(ZulipHttpErrorV1::InvalidCommand);
    }
    let anchor = before_provider_message_id
        .map(message_id)
        .transpose()?
        .map_or_else(|| "newest".to_owned(), |value| value.to_string());
    let include_anchor = before_provider_message_id.is_none();
    let base_path = realm_path(&config.account.realm_url)?;
    Ok(ZulipHttpRequestV1 {
        method: "GET",
        path: format!(
            "{base_path}api/v1/messages?anchor={}&include_anchor={include_anchor}&num_before={limit}&num_after=0&apply_markdown=false&allow_empty_topic_name=true",
            percent_encode(&anchor)
        ),
        form_body: String::new(),
        content_type: "application/x-www-form-urlencoded",
        body: Vec::new(),
    })
}

fn command_account_id(command: &ZulipCommandV1) -> &str {
    match command {
        ZulipCommandV1::SendStream { account_id, .. }
        | ZulipCommandV1::SendDirect { account_id, .. }
        | ZulipCommandV1::UpdateMessage { account_id, .. }
        | ZulipCommandV1::DeleteMessage { account_id, .. }
        | ZulipCommandV1::Reaction { account_id, .. }
        | ZulipCommandV1::SendStreamWithUpload { account_id, .. }
        | ZulipCommandV1::SendDirectWithUpload { account_id, .. } => account_id,
        ZulipCommandV1::DownloadAttachment { account_id, .. } => account_id,
    }
}

fn request(
    method: &'static str,
    path: String,
    fields: &[(&str, String)],
) -> Result<ZulipHttpRequestV1, ZulipHttpErrorV1> {
    let form_body = fields
        .iter()
        .map(|(name, value)| format!("{}={}", percent_encode(name), percent_encode(value)))
        .collect::<Vec<_>>()
        .join("&");
    Ok(ZulipHttpRequestV1 {
        method,
        path,
        body: form_body.as_bytes().to_vec(),
        form_body,
        content_type: "application/x-www-form-urlencoded",
    })
}

pub fn request_for_upload(
    config: &ZulipHttpConfigV1,
    filename: &str,
    bytes: &[u8],
) -> Result<ZulipHttpRequestV1, ZulipHttpErrorV1> {
    let filename = required(filename)?;
    (!bytes.is_empty() && bytes.len() <= 64 * 1024 * 1024)
        .then_some(())
        .ok_or(ZulipHttpErrorV1::InvalidCommand)?;
    let boundary = "makosh-zulip-blob-v1";
    let mut body = format!("--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"{}\"\r\nContent-Type: application/octet-stream\r\n\r\n", filename.replace(['\r', '\n', '"'], "_")).into_bytes();
    body.extend_from_slice(bytes);
    body.extend_from_slice(format!("\r\n--{boundary}--\r\n").as_bytes());
    Ok(ZulipHttpRequestV1 {
        method: "POST",
        path: format!(
            "{}api/v1/user_uploads",
            realm_path(&config.account.realm_url)?
        ),
        form_body: String::new(),
        content_type: "multipart/form-data; boundary=makosh-zulip-blob-v1",
        body,
    })
}

pub fn request_for_user_upload_download(
    config: &ZulipHttpConfigV1,
    upload_path: &str,
) -> Result<ZulipHttpRequestV1, ZulipHttpErrorV1> {
    let upload_path = required(upload_path)?;
    let upload_path = upload_path.strip_prefix('/').unwrap_or(&upload_path);
    (upload_path.starts_with("user_uploads/") && !upload_path.contains(['?', '#']))
        .then_some(())
        .ok_or(ZulipHttpErrorV1::InvalidCommand)?;
    Ok(ZulipHttpRequestV1 {
        method: "GET",
        path: format!("{}{}", realm_path(&config.account.realm_url)?, upload_path),
        form_body: String::new(),
        content_type: "application/octet-stream",
        body: Vec::new(),
    })
}

fn realm_path(realm_url: &str) -> Result<String, ZulipHttpErrorV1> {
    let remainder = realm_url
        .strip_prefix("https://")
        .ok_or(ZulipHttpErrorV1::InvalidConfiguration)?;
    let Some((_, path)) = remainder.split_once('/') else {
        return Ok("/".to_owned());
    };
    Ok(format!("/{}", path.trim_start_matches('/')))
}

fn required(value: &str) -> Result<String, ZulipHttpErrorV1> {
    (!value.trim().is_empty() && !value.contains(['\r', '\n', '\0']))
        .then(|| value.to_owned())
        .ok_or(ZulipHttpErrorV1::InvalidCommand)
}

fn message_id(value: &str) -> Result<i64, ZulipHttpErrorV1> {
    value
        .parse::<i64>()
        .ok()
        .filter(|value| *value > 0)
        .ok_or(ZulipHttpErrorV1::InvalidCommand)
}

fn json_string_array(values: &[String]) -> Result<String, ZulipHttpErrorV1> {
    (!values.is_empty())
        .then_some(())
        .ok_or(ZulipHttpErrorV1::InvalidCommand)?;
    let values = values
        .iter()
        .map(|value| required(value))
        .collect::<Result<Vec<_>, _>>()?;
    serde_json::to_string(&values).map_err(|_| ZulipHttpErrorV1::InvalidCommand)
}

fn percent_encode(value: &str) -> String {
    let mut encoded = String::new();
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~') {
            encoded.push(char::from(byte));
        } else if byte == b' ' {
            encoded.push('+');
        } else {
            encoded.push('%');
            encoded.push(hex(byte >> 4));
            encoded.push(hex(byte & 0x0f));
        }
    }
    encoded
}

fn hex(value: u8) -> char {
    match value {
        0..=9 => char::from_u32(u32::from(b'0' + value)).unwrap_or('0'),
        _ => char::from_u32(u32::from(b'A' + value - 10)).unwrap_or('A'),
    }
}
