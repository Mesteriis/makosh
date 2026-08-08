use makosh_provider_telemost::models::{
    TelemostCohost, TelemostLiveStreamRequest, YandexTelemostConferenceRequest,
};
use serde_json::json;

#[test]
fn conference_request_serializes_only_provider_wire_fields() {
    let request = YandexTelemostConferenceRequest {
        waiting_room_level: Some("moderated".to_owned()),
        live_stream: Some(TelemostLiveStreamRequest {
            access_level: Some("public".to_owned()),
            title: None,
            description: None,
        }),
        cohosts: vec![TelemostCohost {
            email: "cohost@example.test".to_owned(),
        }],
        is_auto_summarization_enabled: Some(true),
        metadata: json!({"local_only": true}),
    };

    assert_eq!(
        serde_json::to_value(request).expect("conference request serializes"),
        json!({
            "waiting_room_level": "moderated",
            "live_stream": {"access_level": "public"},
            "cohosts": [{"email": "cohost@example.test"}],
            "is_auto_summarization_enabled": true
        })
    );
}

#[test]
fn conference_response_deserializes_optional_provider_fields() {
    let conference = serde_json::from_value::<
        makosh_provider_telemost::models::YandexTelemostConference,
    >(json!({
        "id": "conference-1",
        "join_url": "https://telemost.yandex.ru/j/fixture",
        "live_stream": {"watch_url": "https://watch.example.test/fixture"}
    }))
    .expect("provider conference response deserializes");

    assert_eq!(conference.id, "conference-1");
    assert_eq!(
        conference.live_stream.and_then(|stream| stream.watch_url),
        Some("https://watch.example.test/fixture".to_owned())
    );
}
