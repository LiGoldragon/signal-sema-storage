use signal_frame::{HandshakeReply, ProtocolVersion, SIGNAL_FRAME_PROTOCOL_VERSION};
use signal_sema_storage::{ContentHash, FixtureScope, FrameMessage, Request, Wire};
#[test]
fn request_has_stable_typed_binary_encoding() {
    let request = Request::HashFetch {
        hash: ContentHash([7; 32]),
    };
    let bytes = Wire::encode_request(&request).expect("encode");
    let decoded = rkyv::from_bytes::<Request, rkyv::rancor::Error>(&bytes).expect("decode");
    assert_eq!(decoded, request);
}
#[test]
fn fixture_scope_is_explicit() {
    assert_eq!(FixtureScope(1).0, 1);
}

#[test]
fn shared_frame_rejects_unsupported_protocol_versions() {
    let unsupported = ProtocolVersion::new(99, 0, 0);
    assert!(matches!(
        Wire::handshake_reply(unsupported),
        HandshakeReply::Rejected(_)
    ));
    assert!(matches!(
        Wire::handshake_reply(SIGNAL_FRAME_PROTOCOL_VERSION),
        HandshakeReply::Accepted(version) if version == SIGNAL_FRAME_PROTOCOL_VERSION
    ));
}

#[test]
fn request_uses_shared_length_prefixed_exchange_frame() {
    let payload = Wire::encode_request(&Request::HashFetch {
        hash: ContentHash([3; 32]),
    })
    .expect("encode request");
    let frame = Wire::frame_request(payload.clone(), 7).expect("frame request");
    let FrameMessage::Request {
        exchange,
        payload: decoded,
    } = Wire::decode_frame(&frame).expect("decode shared frame")
    else {
        panic!("expected request frame")
    };
    assert_eq!(exchange.sequence.value(), 7);
    assert_eq!(decoded, payload);
}
