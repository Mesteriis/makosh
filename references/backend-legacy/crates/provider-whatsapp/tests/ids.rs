use makosh_provider_whatsapp::ids::{
    whatsapp_web_message_id, whatsapp_web_raw_record_id, whatsapp_web_session_id,
};

#[test]
fn identifiers_are_stable_and_scoped() {
    assert_eq!(
        whatsapp_web_session_id("account-1"),
        whatsapp_web_session_id("account-1")
    );
    assert_ne!(
        whatsapp_web_message_id("account-1", "message-1"),
        whatsapp_web_message_id("account-2", "message-1")
    );
    assert!(
        whatsapp_web_raw_record_id("account-1", "message", "1").starts_with("raw:v5:whatsapp_web:")
    );
}
