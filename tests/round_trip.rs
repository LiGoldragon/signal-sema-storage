use core_logos::EncodedItem;
use core_schema::FixtureFamily;
use signal_frame::{HandshakeReply, ProtocolVersion, SIGNAL_FRAME_PROTOCOL_VERSION};
use signal_sema_storage::{
    BindOutcome, BoundIdentities, ContentHash, DeclaredIdentity, DeclaredKey, DeclaredShape,
    DocumentKey, DocumentKind, DocumentPayload, FixtureScope, FrameMessage, IdentityAssignment,
    IdentityIntent, MintedUniverse, NameTableBytes, Rejection, Reply, Request, SchemaWholeHandle,
    SlotIdentifier, TypeIdentity, Wire,
};
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
fn bind_identities_request_round_trips() {
    let request = Request::BindIdentities {
        whole: SchemaWholeHandle(b"payments/v1".to_vec()),
        declarations: vec![
            DeclaredIdentity {
                key: DeclaredKey(b"Alpha".to_vec()),
                shape: DeclaredShape([1; 32]),
                intent: IdentityIntent::MintOrBind,
            },
            DeclaredIdentity {
                key: DeclaredKey(b"Beta".to_vec()),
                shape: DeclaredShape([2; 32]),
                intent: IdentityIntent::Continue(TypeIdentity(4)),
            },
        ],
    };
    let bytes = Wire::encode_request(&request).expect("encode");
    let decoded = rkyv::from_bytes::<Request, rkyv::rancor::Error>(&bytes).expect("decode");
    assert_eq!(decoded, request);
}

#[test]
fn bound_identities_reply_round_trips() {
    let reply = Reply::IdentitiesBound(BoundIdentities {
        universe: MintedUniverse(9),
        assignments: vec![
            IdentityAssignment {
                key: DeclaredKey(b"Alpha".to_vec()),
                identity: TypeIdentity(0),
                outcome: BindOutcome::Minted,
            },
            IdentityAssignment {
                key: DeclaredKey(b"Beta".to_vec()),
                identity: TypeIdentity(1),
                outcome: BindOutcome::Bound,
            },
        ],
    });
    let bytes = Wire::encode_reply(&reply).expect("encode");
    let decoded = rkyv::from_bytes::<Reply, rkyv::rancor::Error>(&bytes).expect("decode");
    assert_eq!(decoded, reply);
}

#[test]
fn identity_rebind_rejection_round_trips() {
    let reply = Reply::Rejected(Rejection::IdentityRebindRejected {
        identity: TypeIdentity(3),
        bound_shape: DeclaredShape([7; 32]),
        attempted_shape: DeclaredShape([8; 32]),
    });
    let bytes = Wire::encode_reply(&reply).expect("encode");
    let decoded = rkyv::from_bytes::<Reply, rkyv::rancor::Error>(&bytes).expect("decode");
    assert_eq!(decoded, reply);
}
#[test]
fn encoded_schema_and_logos_document_payloads_round_trip() {
    let schema = FixtureFamily::build().schema().clone();
    let payloads = [
        (
            DocumentKind::TypeSchema,
            DocumentPayload::TypeSchema {
                schema,
                names: NameTableBytes(Vec::new()),
            },
        ),
        (
            DocumentKind::Logos,
            DocumentPayload::Logos {
                items: Vec::<EncodedItem>::new(),
                names: NameTableBytes(Vec::new()),
            },
        ),
    ];

    for (kind, payload) in payloads {
        let request = Request::Store {
            key: DocumentKey {
                scope: FixtureScope(1),
                kind,
                slot: SlotIdentifier(0),
            },
            payload,
        };
        let bytes = Wire::encode_request(&request).expect("encode");
        let decoded = rkyv::from_bytes::<Request, rkyv::rancor::Error>(&bytes).expect("decode");
        assert_eq!(decoded, request);
    }
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
