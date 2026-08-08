use serde_json::{Value, json};

pub(super) struct MentionMetadata {
    pub(super) count: i64,
    pub(super) mentions: Vec<String>,
    pub(super) detected_by: &'static str,
}

pub(super) fn derive_mention_metadata(text: &str, tdlib_raw: Option<&Value>) -> MentionMetadata {
    let text_mentions = extract_text_mentions(text);
    let entity_count = tdlib_raw.map(tdlib_mention_entity_count).unwrap_or(0);

    if entity_count > 0 {
        MentionMetadata {
            count: entity_count,
            mentions: text_mentions,
            detected_by: "tdlib_entities",
        }
    } else {
        MentionMetadata {
            count: i64::try_from(text_mentions.len()).unwrap_or(0),
            mentions: text_mentions,
            detected_by: "text_regex",
        }
    }
}

pub(super) fn telegram_public_message_link(
    username: Option<&str>,
    provider_message_id: &str,
) -> Option<String> {
    let username = username
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .filter(|value| {
            value
                .chars()
                .all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
        })?;
    let message_id = provider_message_id
        .rsplit(':')
        .next()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .filter(|value| value.chars().all(|ch| ch.is_ascii_digit()))?;

    Some(format!("https://t.me/{username}/{message_id}"))
}

pub(super) fn derive_tdlib_media_album_metadata(
    raw: &Value,
    provider_chat_id: &str,
) -> Option<(String, String)> {
    let album_id = raw
        .get("media_album_id")
        .and_then(|value| {
            value
                .as_i64()
                .map(|number| number.to_string())
                .or_else(|| value.as_str().map(str::to_owned))
        })
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
        .filter(|value| value != "0")?;
    let chat_id = provider_chat_id.trim();
    let album_key = if chat_id.is_empty() {
        album_id.clone()
    } else {
        format!("{chat_id}:{album_id}")
    };
    Some((album_id, album_key))
}

pub(super) fn derive_tdlib_attachment_metadata(raw: &Value) -> Vec<Value> {
    let Some(content) = raw.get("content") else {
        return Vec::new();
    };
    let Some(content_type) = content.get("@type").and_then(Value::as_str) else {
        return Vec::new();
    };

    match content_type {
        "messagePhoto" => tdlib_photo_attachment(content),
        "messageVideo" => tdlib_file_attachment(
            content,
            "video",
            &["video", "video"],
            tdlib_nested_string(content, &["video", "mime_type"])
                .unwrap_or_else(|| "video/mp4".to_owned()),
            tdlib_nested_string(content, &["video", "file_name"])
                .unwrap_or_else(|| "video.mp4".to_owned()),
        ),
        "messageDocument" => tdlib_file_attachment(
            content,
            "document",
            &["document", "document"],
            tdlib_nested_string(content, &["document", "mime_type"])
                .unwrap_or_else(|| "application/octet-stream".to_owned()),
            tdlib_nested_string(content, &["document", "file_name"])
                .unwrap_or_else(|| "document".to_owned()),
        ),
        "messageAudio" => tdlib_file_attachment(
            content,
            "audio",
            &["audio", "audio"],
            tdlib_nested_string(content, &["audio", "mime_type"])
                .unwrap_or_else(|| "audio/mpeg".to_owned()),
            tdlib_nested_string(content, &["audio", "file_name"])
                .unwrap_or_else(|| "audio.mp3".to_owned()),
        ),
        "messageVoiceNote" => tdlib_file_attachment(
            content,
            "voice",
            &["voice_note", "voice"],
            tdlib_nested_string(content, &["voice_note", "mime_type"])
                .unwrap_or_else(|| "audio/ogg".to_owned()),
            "voice-note.ogg".to_owned(),
        ),
        "messageSticker" => tdlib_file_attachment(
            content,
            "sticker",
            &["sticker", "sticker"],
            tdlib_sticker_mime_type(content),
            tdlib_sticker_filename(content),
        ),
        "messageAnimation" => tdlib_file_attachment(
            content,
            "animation",
            &["animation", "animation"],
            tdlib_nested_string(content, &["animation", "mime_type"])
                .unwrap_or_else(|| "video/mp4".to_owned()),
            tdlib_nested_string(content, &["animation", "file_name"])
                .unwrap_or_else(|| "animation.mp4".to_owned()),
        ),
        "messageVideoNote" => tdlib_file_attachment(
            content,
            "video_note",
            &["video_note", "video"],
            "video/mp4".to_owned(),
            "video-note.mp4".to_owned(),
        ),
        _ => None,
    }
    .into_iter()
    .collect()
}

pub(super) fn derive_tdlib_structured_evidence(raw: &Value) -> Vec<(String, Value)> {
    let Some(content) = raw.get("content") else {
        return Vec::new();
    };
    let Some(content_type) = content.get("@type").and_then(Value::as_str) else {
        return Vec::new();
    };

    match content_type {
        "messagePoll" => tdlib_poll_evidence(content)
            .map(|value| vec![("telegram_poll".to_owned(), value)])
            .unwrap_or_default(),
        "messageLocation" => tdlib_location_evidence(content)
            .map(|value| vec![("telegram_location".to_owned(), value)])
            .unwrap_or_default(),
        "messageVenue" => tdlib_venue_evidence(content)
            .map(|value| vec![("telegram_location".to_owned(), value)])
            .unwrap_or_default(),
        "messageContact" => tdlib_contact_evidence(content)
            .map(|value| vec![("telegram_contact_card".to_owned(), value)])
            .unwrap_or_default(),
        "messageChatAddMembers"
        | "messageChatJoinByLink"
        | "messageChatJoinByRequest"
        | "messageChatDeleteMember" => tdlib_join_leave_evidence(content, content_type)
            .map(|value| vec![("telegram_join_leave".to_owned(), value)])
            .unwrap_or_default(),
        _ => Vec::new(),
    }
}

fn extract_text_mentions(text: &str) -> Vec<String> {
    let mut mentions = Vec::new();
    let chars: Vec<char> = text.chars().collect();
    let mut index = 0usize;
    while index < chars.len() {
        if chars[index] != '@' {
            index += 1;
            continue;
        }
        let mut end = index + 1;
        while end < chars.len() && is_telegram_mention_char(chars[end]) {
            end += 1;
        }
        if end.saturating_sub(index) >= 3 {
            let mention: String = chars[index..end].iter().collect();
            if !mentions.iter().any(|existing| existing == &mention) {
                mentions.push(mention);
            }
        }
        index = end;
    }
    mentions
}

fn is_telegram_mention_char(value: char) -> bool {
    value.is_ascii_alphanumeric() || value == '_'
}

fn tdlib_mention_entity_count(raw: &Value) -> i64 {
    tdlib_formatted_text_entities(raw)
        .into_iter()
        .flat_map(|entities| entities.iter())
        .filter(|entity| {
            matches!(
                entity
                    .get("type")
                    .and_then(|value| value.get("@type"))
                    .and_then(Value::as_str),
                Some("textEntityTypeMention" | "textEntityTypeMentionName")
            )
        })
        .count() as i64
}

fn tdlib_formatted_text_entities(raw: &Value) -> Vec<&Vec<Value>> {
    let mut entities = Vec::new();
    if let Some(content) = raw.get("content") {
        for key in ["text", "caption"] {
            if let Some(array) = content
                .get(key)
                .and_then(|value| value.get("entities"))
                .and_then(Value::as_array)
            {
                entities.push(array);
            }
        }
    }
    entities
}

fn tdlib_poll_evidence(content: &Value) -> Option<Value> {
    let poll = content.get("poll")?;
    let question = tdlib_nested_string(poll, &["question", "text"])
        .or_else(|| tdlib_nested_string(poll, &["question"]))?;
    let options = poll
        .get("options")
        .and_then(Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(|option| {
                    tdlib_nested_string(option, &["text", "text"])
                        .or_else(|| tdlib_nested_string(option, &["text"]))
                        .map(|text| {
                            json!({
                                "text": text,
                                "voter_count": option.get("voter_count").and_then(Value::as_i64),
                            })
                        })
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    Some(json!({
        "question": question,
        "options": options,
        "total_voter_count": poll.get("total_voter_count").and_then(Value::as_i64),
        "is_closed": poll.get("is_closed").and_then(Value::as_bool).unwrap_or(false),
        "poll_type": poll.get("type").and_then(|value| value.get("@type")).and_then(Value::as_str),
    }))
}

fn tdlib_location_evidence(content: &Value) -> Option<Value> {
    let location = content.get("location")?;
    Some(json!({
        "kind": "location",
        "latitude": location.get("latitude").and_then(Value::as_f64)?,
        "longitude": location.get("longitude").and_then(Value::as_f64)?,
        "horizontal_accuracy": location.get("horizontal_accuracy").and_then(Value::as_f64),
    }))
}

fn tdlib_venue_evidence(content: &Value) -> Option<Value> {
    let venue = content.get("venue")?;
    let location = venue.get("location")?;
    Some(json!({
        "kind": "venue",
        "title": venue.get("title").and_then(Value::as_str),
        "address": venue.get("address").and_then(Value::as_str),
        "latitude": location.get("latitude").and_then(Value::as_f64)?,
        "longitude": location.get("longitude").and_then(Value::as_f64)?,
        "provider": venue.get("provider").and_then(Value::as_str),
        "venue_id": venue.get("id").and_then(Value::as_str),
    }))
}

fn tdlib_contact_evidence(content: &Value) -> Option<Value> {
    let contact = content.get("contact")?;
    Some(json!({
        "phone_number": contact.get("phone_number").and_then(Value::as_str),
        "first_name": contact.get("first_name").and_then(Value::as_str),
        "last_name": contact.get("last_name").and_then(Value::as_str),
        "user_id": contact.get("user_id").and_then(Value::as_i64),
        "vcard": contact.get("vcard").and_then(Value::as_str),
    }))
}

fn tdlib_join_leave_evidence(content: &Value, content_type: &str) -> Option<Value> {
    match content_type {
        "messageChatAddMembers" => Some(json!({
            "action": "join",
            "source": "add_members",
            "user_ids": content.get("member_user_ids").and_then(Value::as_array).cloned().unwrap_or_default(),
        })),
        "messageChatJoinByLink" => Some(json!({
            "action": "join",
            "source": "join_by_link",
        })),
        "messageChatJoinByRequest" => Some(json!({
            "action": "join",
            "source": "join_by_request",
        })),
        "messageChatDeleteMember" => Some(json!({
            "action": "leave",
            "source": "delete_member",
            "user_id": content.get("user_id").and_then(Value::as_i64),
        })),
        _ => None,
    }
}

fn tdlib_file_attachment(
    content: &Value,
    attachment_type: &str,
    file_path: &[&str],
    content_type: String,
    filename: String,
) -> Option<Value> {
    let file = tdlib_nested_value(content, file_path)?;
    tdlib_attachment_from_file(content, attachment_type, file, content_type, filename)
}

fn tdlib_photo_attachment(content: &Value) -> Option<Value> {
    let file = tdlib_largest_photo_file(content)?;
    let file_id = file.get("id").and_then(Value::as_i64)?;

    tdlib_attachment_from_file(
        content,
        "photo",
        file,
        "image/jpeg".to_owned(),
        format!("photo-{file_id}.jpg"),
    )
}

fn tdlib_largest_photo_file(content: &Value) -> Option<&Value> {
    content
        .get("photo")?
        .get("sizes")?
        .as_array()?
        .iter()
        .filter_map(|size| {
            let file = size.get("photo")?;
            let file_id = file.get("id").and_then(Value::as_i64)?;
            (file_id > 0).then_some((
                file,
                size.get("width")
                    .and_then(Value::as_u64)
                    .unwrap_or_default()
                    .saturating_mul(
                        size.get("height")
                            .and_then(Value::as_u64)
                            .unwrap_or_default(),
                    ),
            ))
        })
        .max_by_key(|(_, area)| *area)
        .map(|(file, _)| file)
}

fn tdlib_attachment_from_file(
    content: &Value,
    attachment_type: &str,
    file: &Value,
    content_type: String,
    filename: String,
) -> Option<Value> {
    let tdlib_file_id = file.get("id").and_then(Value::as_i64)?;
    if tdlib_file_id <= 0 {
        return None;
    }
    let size = file.get("size").and_then(Value::as_i64);
    let provider_attachment_id = format!("tdlib:{attachment_type}:{tdlib_file_id}");
    let mut attachment = json!({
        "id": provider_attachment_id,
        "attachment_id": provider_attachment_id,
        "attachment_type": attachment_type,
        "content_type": content_type,
        "filename": filename,
        "tdlib_file_id": tdlib_file_id,
        "download_state": "remote",
        "metadata": {
            "tdlib_content_type": content.get("@type").and_then(Value::as_str),
        },
    });
    if let (Some(object), Some(size)) = (attachment.as_object_mut(), size) {
        object.insert("size".to_owned(), json!(size));
    }
    if attachment_type == "sticker"
        && let Some(emoji) = tdlib_nested_string(content, &["sticker", "emoji"])
        && let Some(metadata) = attachment
            .get_mut("metadata")
            .and_then(Value::as_object_mut)
    {
        metadata.insert("emoji".to_owned(), Value::String(emoji));
    }
    Some(attachment)
}

fn tdlib_sticker_mime_type(content: &Value) -> String {
    match tdlib_nested_string(content, &["sticker", "format", "@type"]).as_deref() {
        Some("stickerFormatTgs") => "application/x-tgsticker".to_owned(),
        Some("stickerFormatWebm") => "video/webm".to_owned(),
        _ => "image/webp".to_owned(),
    }
}

fn tdlib_sticker_filename(content: &Value) -> String {
    let extension = match tdlib_nested_string(content, &["sticker", "format", "@type"]).as_deref() {
        Some("stickerFormatTgs") => "tgs",
        Some("stickerFormatWebm") => "webm",
        _ => "webp",
    };
    let emoji = tdlib_nested_string(content, &["sticker", "emoji"])
        .filter(|value| {
            value
                .chars()
                .all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
        })
        .unwrap_or_else(|| "sticker".to_owned());
    format!("{emoji}.{extension}")
}

fn tdlib_nested_string(value: &Value, path: &[&str]) -> Option<String> {
    tdlib_nested_value(value, path)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn tdlib_nested_value<'a>(value: &'a Value, path: &[&str]) -> Option<&'a Value> {
    let mut current = value;
    for segment in path {
        current = current.get(*segment)?;
    }
    Some(current)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{
        derive_tdlib_attachment_metadata, derive_tdlib_media_album_metadata,
        derive_tdlib_structured_evidence,
    };

    #[test]
    fn derives_sticker_animation_and_video_note_attachment_metadata_from_tdlib_raw() {
        let sticker = derive_tdlib_attachment_metadata(&json!({
            "@type": "message",
            "content": {
                "@type": "messageSticker",
                "sticker": {
                    "@type": "sticker",
                    "emoji": "ok",
                    "format": {"@type": "stickerFormatWebp"},
                    "sticker": {"@type": "file", "id": 701, "size": 4096}
                }
            }
        }));
        let animation = derive_tdlib_attachment_metadata(&json!({
            "@type": "message",
            "content": {
                "@type": "messageAnimation",
                "animation": {
                    "@type": "animation",
                    "file_name": "loop.mp4",
                    "mime_type": "video/mp4",
                    "animation": {"@type": "file", "id": 702, "size": 8192}
                }
            }
        }));
        let video_note = derive_tdlib_attachment_metadata(&json!({
            "@type": "message",
            "content": {
                "@type": "messageVideoNote",
                "video_note": {
                    "@type": "videoNote",
                    "video": {"@type": "file", "id": 703, "size": 16384}
                }
            }
        }));

        assert_eq!(sticker[0]["attachment_type"], json!("sticker"));
        assert_eq!(sticker[0]["tdlib_file_id"], json!(701));
        assert_eq!(sticker[0]["content_type"], json!("image/webp"));
        assert_eq!(sticker[0]["metadata"]["emoji"], json!("ok"));
        assert_eq!(animation[0]["attachment_type"], json!("animation"));
        assert_eq!(animation[0]["filename"], json!("loop.mp4"));
        assert_eq!(animation[0]["tdlib_file_id"], json!(702));
        assert_eq!(video_note[0]["attachment_type"], json!("video_note"));
        assert_eq!(video_note[0]["content_type"], json!("video/mp4"));
        assert_eq!(video_note[0]["tdlib_file_id"], json!(703));
    }

    #[test]
    fn derives_standard_photo_video_document_audio_and_voice_attachment_metadata() {
        let photo = derive_tdlib_attachment_metadata(&json!({
            "content": {
                "@type": "messagePhoto",
                "photo": {
                    "sizes": [
                        {"width": 90, "height": 90, "photo": {"id": 801, "size": 1024}},
                        {"width": 1280, "height": 720, "photo": {"id": 802, "size": 16384}}
                    ]
                }
            }
        }));
        let video = derive_tdlib_attachment_metadata(&json!({
            "content": {
                "@type": "messageVideo",
                "video": {"file_name": "clip.webm", "mime_type": "video/webm", "video": {"id": 803, "size": 32768}}
            }
        }));
        let document = derive_tdlib_attachment_metadata(&json!({
            "content": {
                "@type": "messageDocument",
                "document": {"file_name": "brief.pdf", "mime_type": "application/pdf", "document": {"id": 804, "size": 4096}}
            }
        }));
        let audio = derive_tdlib_attachment_metadata(&json!({
            "content": {
                "@type": "messageAudio",
                "audio": {"file_name": "song.ogg", "mime_type": "audio/ogg", "audio": {"id": 805, "size": 8192}}
            }
        }));
        let voice = derive_tdlib_attachment_metadata(&json!({
            "content": {
                "@type": "messageVoiceNote",
                "voice_note": {"mime_type": "audio/ogg", "voice": {"id": 806, "size": 2048}}
            }
        }));

        assert_eq!(photo[0]["attachment_type"], json!("photo"));
        assert_eq!(photo[0]["tdlib_file_id"], json!(802));
        assert_eq!(photo[0]["filename"], json!("photo-802.jpg"));
        assert_eq!(video[0]["attachment_type"], json!("video"));
        assert_eq!(video[0]["content_type"], json!("video/webm"));
        assert_eq!(document[0]["attachment_type"], json!("document"));
        assert_eq!(document[0]["filename"], json!("brief.pdf"));
        assert_eq!(audio[0]["attachment_type"], json!("audio"));
        assert_eq!(audio[0]["tdlib_file_id"], json!(805));
        assert_eq!(voice[0]["attachment_type"], json!("voice"));
        assert_eq!(voice[0]["content_type"], json!("audio/ogg"));
    }

    #[test]
    fn derives_poll_location_venue_contact_and_join_leave_structured_evidence_from_tdlib_raw() {
        let poll = derive_tdlib_structured_evidence(&json!({
            "content": {
                "@type": "messagePoll",
                "poll": {
                    "question": {"text": "Pick one"},
                    "options": [
                        {"text": {"text": "A"}, "voter_count": 2},
                        {"text": {"text": "B"}, "voter_count": 3}
                    ],
                    "total_voter_count": 5,
                    "is_closed": false,
                    "type": {"@type": "pollTypeRegular"}
                }
            }
        }));
        let location = derive_tdlib_structured_evidence(&json!({
            "content": {
                "@type": "messageLocation",
                "location": {"latitude": 41.3874, "longitude": 2.1686, "horizontal_accuracy": 12.5}
            }
        }));
        let venue = derive_tdlib_structured_evidence(&json!({
            "content": {
                "@type": "messageVenue",
                "venue": {
                    "title": "Cafe Макошь",
                    "address": "Local street",
                    "provider": "foursquare",
                    "id": "venue-1",
                    "location": {"latitude": 41.0, "longitude": 2.0}
                }
            }
        }));
        let contact = derive_tdlib_structured_evidence(&json!({
            "content": {
                "@type": "messageContact",
                "contact": {
                    "phone_number": "+34123456789",
                    "first_name": "Ada",
                    "last_name": "Lovelace",
                    "user_id": 42,
                    "vcard": "BEGIN:VCARD"
                }
            }
        }));
        let join = derive_tdlib_structured_evidence(&json!({
            "content": {
                "@type": "messageChatAddMembers",
                "member_user_ids": [7, 8]
            }
        }));
        let leave = derive_tdlib_structured_evidence(&json!({
            "content": {
                "@type": "messageChatDeleteMember",
                "user_id": 9
            }
        }));

        assert_eq!(poll[0].0, "telegram_poll");
        assert_eq!(poll[0].1["question"], json!("Pick one"));
        assert_eq!(poll[0].1["options"][1]["text"], json!("B"));
        assert_eq!(location[0].0, "telegram_location");
        assert_eq!(location[0].1["kind"], json!("location"));
        assert_eq!(location[0].1["latitude"], json!(41.3874));
        assert_eq!(venue[0].1["kind"], json!("venue"));
        assert_eq!(venue[0].1["title"], json!("Cafe Макошь"));
        assert_eq!(contact[0].0, "telegram_contact_card");
        assert_eq!(contact[0].1["first_name"], json!("Ada"));
        assert_eq!(contact[0].1["phone_number"], json!("+34123456789"));
        assert_eq!(join[0].0, "telegram_join_leave");
        assert_eq!(join[0].1["action"], json!("join"));
        assert_eq!(join[0].1["user_ids"], json!([7, 8]));
        assert_eq!(leave[0].0, "telegram_join_leave");
        assert_eq!(leave[0].1["action"], json!("leave"));
        assert_eq!(leave[0].1["user_id"], json!(9));
    }

    #[test]
    fn derives_media_album_projection_metadata_from_tdlib_raw() {
        assert_eq!(
            derive_tdlib_media_album_metadata(
                &json!({"@type": "message", "media_album_id": 998877}),
                "-100123"
            ),
            Some(("998877".to_owned(), "-100123:998877".to_owned()))
        );
        assert_eq!(
            derive_tdlib_media_album_metadata(
                &json!({"@type": "message", "media_album_id": "album-42"}),
                "chat-a"
            ),
            Some(("album-42".to_owned(), "chat-a:album-42".to_owned()))
        );
        assert_eq!(
            derive_tdlib_media_album_metadata(
                &json!({"@type": "message", "media_album_id": 0}),
                "chat-a"
            ),
            None
        );
    }
}
