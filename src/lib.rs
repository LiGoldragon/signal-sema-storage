//! Typed binary contract for the prototype's central, standalone Sema daemon.
use core_logos::CoreItem;
use core_schema::CoreSchema;
use name_table::Identifier;
use rkyv::{Archive, Deserialize, Serialize};

pub const WIRE_VERSION: u16 = 1;

#[derive(Archive, Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FixtureScope(pub u64);
#[derive(Archive, Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SlotIdentifier(pub u64);
#[derive(Archive, Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Version(pub u64);
#[derive(Archive, Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Snapshot(pub u64);
#[derive(Archive, Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ContentHash(pub [u8; 32]);
#[derive(Archive, Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SubscriptionIdentifier(pub u64);
#[derive(Archive, Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct IdentifierBlock { pub first: u32, pub length: u32 }

#[derive(Archive, Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DocumentKind { TypeSchema, SignalContract, NexusRuntime, SemaStorage, Nomos, Logos }

#[derive(Archive, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct DocumentKey { pub scope: FixtureScope, pub kind: DocumentKind, pub slot: SlotIdentifier }

#[derive(Archive, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct NamedDeclaration { pub identifier: Identifier, pub members: Vec<Identifier> }

#[derive(Archive, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct NameTableBytes(pub Vec<u8>);

#[derive(Archive, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct DeclarationRoot { pub kind: DocumentKind, pub declarations: Vec<NamedDeclaration>, pub names: NameTableBytes }

/// The minimal prototype has one real, fixture-scoped macro package. The enum is
/// its stable typed identity; the Nomos daemon reconstructs `MacroPackage::wire_fixture()`.
#[derive(Archive, Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum NomosPackage { WireFixture }

#[derive(Archive, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum DocumentPayload {
    TypeSchema { schema: CoreSchema, names: NameTableBytes },
    SignalContract(DeclarationRoot),
    NexusRuntime(DeclarationRoot),
    SemaStorage(DeclarationRoot),
    Nomos(NomosPackage),
    Logos { items: Vec<CoreItem>, names: NameTableBytes },
}

impl DocumentPayload {
    pub fn kind(&self) -> DocumentKind { match self { Self::TypeSchema{..}=>DocumentKind::TypeSchema, Self::SignalContract(_)=>DocumentKind::SignalContract, Self::NexusRuntime(_)=>DocumentKind::NexusRuntime, Self::SemaStorage(_)=>DocumentKind::SemaStorage, Self::Nomos(_)=>DocumentKind::Nomos, Self::Logos{..}=>DocumentKind::Logos } }
    pub fn content_hash(&self) -> Result<ContentHash, CodecError> {
        let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(self).map_err(|e| CodecError::Encode(e.to_string()))?;
        Ok(ContentHash(*blake3::hash(&bytes).as_bytes()))
    }
}

#[derive(Archive, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct StoredDocument { pub key: DocumentKey, pub version: Version, pub hash: ContentHash, pub payload: DocumentPayload }
#[derive(Archive, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct SlotSummary { pub key: DocumentKey, pub version: Version, pub hash: ContentHash }
#[derive(Archive, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct ChangeEvent { pub subscription: SubscriptionIdentifier, pub snapshot: Snapshot, pub document: SlotSummary }

#[derive(Archive, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum Request {
    Store { key: DocumentKey, payload: DocumentPayload },
    Fetch { key: DocumentKey, version: Option<Version> },
    List { scope: FixtureScope, kind: Option<DocumentKind> },
    HashFetch { hash: ContentHash },
    Snapshot { scope: FixtureScope },
    Subscribe { scope: FixtureScope, kind: Option<DocumentKind> },
    AllocateIdentifiers { scope: FixtureScope, count: u32 },
}
#[derive(Archive, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum Reply {
    Stored(SlotSummary), Document(Option<StoredDocument>), Listed(Vec<SlotSummary>),
    Snapshotted(Snapshot), Subscribed { identifier: SubscriptionIdentifier, initial: Vec<SlotSummary> },
    IdentifiersAllocated(IdentifierBlock), Rejected(Rejection), Event(ChangeEvent),
}
#[derive(Archive, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum Rejection { InvalidKind, NotFound, CountZero, IncompatibleWireVersion, Internal }

#[derive(Debug, thiserror::Error)]
pub enum CodecError { #[error("encode: {0}")] Encode(String), #[error("decode: {0}")] Decode(String), #[error("frame too large")] FrameTooLarge }

pub struct Wire;
impl Wire {
    pub fn encode_request(value: &Request) -> Result<Vec<u8>, CodecError> {
        rkyv::to_bytes::<rkyv::rancor::Error>(value).map(|b| b.to_vec()).map_err(|e| CodecError::Encode(e.to_string()))
    }
    pub fn encode_reply(value: &Reply) -> Result<Vec<u8>, CodecError> {
        rkyv::to_bytes::<rkyv::rancor::Error>(value).map(|b| b.to_vec()).map_err(|e| CodecError::Encode(e.to_string()))
    }
}
