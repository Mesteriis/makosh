use makosh_provider_telegram::tdlib::{chats, messages, topics, types::TdlibMediaKind};
use serde_json::json;

#[test]
fn text_reply_and_edit_use_tdlib_formatted_text_contract() {
    let send = messages::send_text(10, " hello ", "send-1").expect("send request");
    assert_eq!(send["input_message_content"]["text"]["text"], "hello");
    assert_eq!(send["input_message_content"]["clear_draft"], true);

    let reply = messages::send_reply(10, 11, "reply", "reply-1").expect("reply request");
    assert_eq!(reply["reply_to"]["message_id"], 11);

    let edit = messages::edit_text(10, 11, "updated", "edit-1").expect("edit request");
    assert_eq!(edit["@type"], "editMessageText");
    assert!(messages::edit_text(10, 11, " ", "edit-1").is_err());
}

#[test]
fn media_contract_rejects_empty_paths_and_keeps_document_metadata() {
    let media = messages::send_media(
        10,
        TdlibMediaKind::Document,
        "/tmp/makosh/upload.pdf",
        Some("Document caption"),
        Some("upload.pdf"),
        "media-1",
    )
    .expect("media request");
    assert_eq!(
        media["input_message_content"]["@type"],
        "inputMessageDocument"
    );
    assert_eq!(
        media["input_message_content"]["document"]["path"],
        "/tmp/makosh/upload.pdf"
    );
    assert_eq!(
        media["input_message_content"]["caption"]["text"],
        "Document caption"
    );
    assert!(messages::send_media(10, TdlibMediaKind::Photo, " ", None, None, "media-2").is_err());
}

#[test]
fn message_mutation_commands_preserve_tdlib_shape() {
    let deletion = messages::delete_messages(10, &[11, 12], true, "delete-1");
    assert_eq!(deletion["@type"], "deleteMessages");
    assert_eq!(deletion["message_ids"], json!([11, 12]));

    let reaction = messages::add_reaction(10, 11, "👍", "react-1");
    assert_eq!(reaction["reaction_type"]["emoji"], "👍");
    assert_eq!(reaction["update_recent_reactions"], true);
    let unreaction = messages::remove_reaction(10, 11, "👍", "unreact-1");
    assert_eq!(unreaction["@type"], "removeMessageReaction");

    let pinned = messages::pin_message(10, 11, false, "pin-1");
    assert_eq!(pinned["only_for_self"], false);
    assert_eq!(
        messages::unpin_message(10, 11, "unpin-1")["@type"],
        "unpinChatMessage"
    );
    assert_eq!(
        messages::view_messages(10, &[11], true, "view-1")["force_read"],
        true
    );
    assert_eq!(
        messages::forward_message(10, 20, 11, "forward-1")["from_chat_id"],
        20
    );
}

#[test]
fn chat_query_commands_clamp_limits_and_normalize_extras() {
    assert_eq!(chats::load_chats(500, " load ")["limit"], 100);
    assert_eq!(chats::get_chats(0, " get ")["limit"], 1);
    assert_eq!(chats::get_chat(10, "chat")["@type"], "getChat");
    assert_eq!(chats::get_basic_group(11, "group")["basic_group_id"], 11);
    assert_eq!(
        chats::get_basic_group_full_info(11, "group")["@type"],
        "getBasicGroupFullInfo"
    );
    assert_eq!(chats::get_chat_folder(7, " folder ")["@extra"], "folder");

    let history = chats::get_chat_history(10, Some(11), 500, true, "history");
    assert_eq!(history["from_message_id"], 11);
    assert_eq!(history["limit"], 100);
    assert_eq!(chats::download_file(42, 99, "download")["priority"], 32);
    assert_eq!(
        chats::search_messages(" hello ", 0, "search")["query"],
        "hello"
    );
    assert_eq!(
        chats::search_chat_messages(10, " hello ", 0, "search")["limit"],
        1
    );
}

#[test]
fn chat_state_and_membership_commands_preserve_provider_contract() {
    assert_eq!(
        chats::toggle_marked_as_unread(10, true, "unread")["is_marked_as_unread"],
        true
    );
    assert_eq!(
        chats::add_chat_to_list(10, true, "archive")["chat_list"]["@type"],
        "chatListArchive"
    );
    assert_eq!(
        chats::add_chat_to_folder(10, 7, "folder")["chat_list"]["chat_folder_id"],
        7
    );
    assert_eq!(
        chats::set_chat_mute(10, true, "mute")["notification_settings"]["mute_for"],
        31_708_800
    );
    assert_eq!(chats::join_chat(10, "join")["@type"], "joinChat");
    assert_eq!(chats::leave_chat(10, "leave")["@type"], "leaveChat");

    let members = chats::get_supergroup_members(10, -1, 500, "members");
    assert_eq!(members["filter"]["@type"], "supergroupMembersFilterRecent");
    assert_eq!(members["offset"], 0);
    assert_eq!(members["limit"], 100);
    assert_eq!(
        chats::get_supergroup_administrators(10, 0, 20, "admins")["filter"]["@type"],
        "supergroupMembersFilterAdministrators"
    );
}

#[test]
fn forum_topic_commands_validate_titles_and_preserve_shape() {
    let topics_list = topics::get_forum_topics(10, 500, "topics");
    assert_eq!(topics_list["limit"], 100);
    let created =
        topics::create_forum_topic(10, " Release planning ", "create").expect("topic request");
    assert_eq!(created["name"], "Release planning");
    assert!(topics::create_forum_topic(10, " ", "create").is_err());
    assert_eq!(
        topics::toggle_forum_topic_closed(10, 11, true, "close")["is_closed"],
        true
    );
}

#[test]
fn media_kind_accepts_provider_aliases() {
    assert!(matches!(
        TdlibMediaKind::try_from("voice_note"),
        Ok(TdlibMediaKind::Voice)
    ));
    assert!(matches!(
        TdlibMediaKind::try_from("gif"),
        Ok(TdlibMediaKind::Animation)
    ));
    assert!(TdlibMediaKind::try_from("binary").is_err());
}
