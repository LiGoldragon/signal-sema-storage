use signal_sema_storage::{ContentHash, FixtureScope, Request, Wire};
#[test]
fn request_has_stable_typed_binary_encoding() {
 let request=Request::HashFetch{hash:ContentHash([7;32])};
 let bytes=Wire::encode_request(&request).expect("encode");
 let decoded=rkyv::from_bytes::<Request,rkyv::rancor::Error>(&bytes).expect("decode");
 assert_eq!(decoded,request);
}
#[test]
fn fixture_scope_is_explicit() { assert_eq!(FixtureScope(1).0,1); }
