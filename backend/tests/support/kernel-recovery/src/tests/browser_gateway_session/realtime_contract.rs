use super::*;

#[test]
fn client_realtime_contract_rejects_unversioned_and_sse_unsafe_cursors() {
    let mut frame = ClientRealtimeFrameV1 {
        frame: Some(Frame::Event(ClientRealtimeEventV1 {
            event_id: vec![1; 16],
            cursor: "cursor-1".to_owned(),
            contract_name: "makosh.client.status".to_owned(),
            contract_version: 1,
            event_kind: "status_changed".to_owned(),
            occurred_at_unix_millis: 1,
            causation_id: String::new(),
            correlation_id: String::new(),
            trace_id: String::new(),
            payload: vec![2; 32],
        })),
    };
    assert!(validate_client_realtime_frame(&frame).is_ok());
    set_realtime_frame_version(&mut frame, 0);
    assert!(validate_client_realtime_frame(&frame).is_err());
    set_unsafe_realtime_cursor(&mut frame);
    assert!(validate_client_realtime_frame(&frame).is_err());
}

fn set_realtime_frame_version(frame: &mut ClientRealtimeFrameV1, version: u32) {
    let Some(Frame::Event(event)) = frame.frame.as_mut() else {
        panic!("fixture contains a client event");
    };
    event.contract_version = version;
}

fn set_unsafe_realtime_cursor(frame: &mut ClientRealtimeFrameV1) {
    let Some(Frame::Event(event)) = frame.frame.as_mut() else {
        panic!("fixture contains a client event");
    };
    event.contract_version = 1;
    event.cursor = "cursor-1\nevent: injected".to_owned();
}
