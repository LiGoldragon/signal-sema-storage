//! Typed binary contract for the prototype's central, standalone Sema daemon.
use std::collections::BTreeSet;

use core_logos::CoreItem;
use core_schema::CoreSchema;
use name_table::Identifier;
use rkyv::{Archive, Deserialize, Serialize};

pub const WIRE_VERSION: u16 = 2;

#[derive(
    Archive, Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
pub struct FixtureScope(pub u64);
#[derive(
    Archive, Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
pub struct SlotIdentifier(pub u64);
#[derive(
    Archive, Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
pub struct Version(pub u64);
#[derive(
    Archive, Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
pub struct Snapshot(pub u64);
#[derive(
    Archive, Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
pub struct ContentHash(pub [u8; 32]);
#[derive(
    Archive, Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
pub struct SubscriptionIdentifier(pub u64);
#[derive(Archive, Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct IdentifierBlock {
    pub first: u32,
    pub length: u32,
}

#[derive(
    Archive, Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
pub enum DocumentKind {
    TypeSchema,
    SignalContract,
    NexusRuntime,
    SemaStorage,
    Nomos,
    Logos,
}

#[derive(Archive, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct DocumentKey {
    pub scope: FixtureScope,
    pub kind: DocumentKind,
    pub slot: SlotIdentifier,
}

#[derive(Archive, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct NameTableBytes(pub Vec<u8>);

#[derive(Archive, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct StreamDeclaration {
    pub stream: Identifier,
}
#[derive(Archive, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct OpensRelation {
    pub operation: Identifier,
    pub stream: Identifier,
}
#[derive(Archive, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct BelongsRelation {
    pub stream: Identifier,
    pub contract: Identifier,
}
#[derive(Archive, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct SignalContractRoot {
    pub contract: Identifier,
    pub streams: Vec<StreamDeclaration>,
    pub opens: Vec<OpensRelation>,
    pub belongs: Vec<BelongsRelation>,
    pub names: NameTableBytes,
}

impl SignalContractRoot {
    pub fn validate(&self) -> Result<(), RootViolation> {
        let streams: BTreeSet<_> = self.streams.iter().map(|entry| entry.stream).collect();
        if streams.len() != self.streams.len() {
            return Err(RootViolation::DuplicateStream);
        }
        if self
            .opens
            .iter()
            .any(|relation| !streams.contains(&relation.stream))
        {
            return Err(RootViolation::UnknownOpenedStream);
        }
        if self.belongs.iter().any(|relation| {
            !streams.contains(&relation.stream) || relation.contract != self.contract
        }) {
            return Err(RootViolation::InvalidBelongsRelation);
        }
        Ok(())
    }
}

#[derive(Archive, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct NexusActorDeclaration {
    pub actor: Identifier,
}
#[derive(Archive, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct NexusRoute {
    pub sender: Identifier,
    pub receiver: Identifier,
}
#[derive(Archive, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct NexusRuntimeRoot {
    pub actors: Vec<NexusActorDeclaration>,
    pub routes: Vec<NexusRoute>,
    pub names: NameTableBytes,
}
impl NexusRuntimeRoot {
    pub fn validate(&self) -> Result<(), RootViolation> {
        let actors: BTreeSet<_> = self.actors.iter().map(|entry| entry.actor).collect();
        if actors.len() != self.actors.len() {
            return Err(RootViolation::DuplicateActor);
        }
        if self
            .routes
            .iter()
            .any(|route| !actors.contains(&route.sender) || !actors.contains(&route.receiver))
        {
            return Err(RootViolation::UnknownRouteActor);
        }
        Ok(())
    }
}

#[derive(Archive, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct FamilyDeclaration {
    pub family: Identifier,
    pub layout_version: u32,
}
#[derive(Archive, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct SemaStorageRoot {
    pub families: Vec<FamilyDeclaration>,
    pub names: NameTableBytes,
}
impl SemaStorageRoot {
    pub fn validate(&self) -> Result<(), RootViolation> {
        let families: BTreeSet<_> = self.families.iter().map(|entry| entry.family).collect();
        if families.len() != self.families.len() {
            return Err(RootViolation::DuplicateFamily);
        }
        if self
            .families
            .iter()
            .any(|family| family.layout_version == 0)
        {
            return Err(RootViolation::ZeroLayoutVersion);
        }
        Ok(())
    }
}

#[derive(Archive, Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum RootViolation {
    DuplicateStream,
    UnknownOpenedStream,
    InvalidBelongsRelation,
    DuplicateActor,
    UnknownRouteActor,
    DuplicateFamily,
    ZeroLayoutVersion,
}

/// The prototype has one real, fixture-scoped macro package. This stable typed
/// identity reconstructs `MacroPackage::wire_fixture()` in the Nomos daemon.
#[derive(Archive, Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum NomosPackage {
    WireFixture,
}

#[derive(Archive, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum DocumentPayload {
    TypeSchema {
        schema: CoreSchema,
        names: NameTableBytes,
    },
    SignalContract(SignalContractRoot),
    NexusRuntime(NexusRuntimeRoot),
    SemaStorage(SemaStorageRoot),
    Nomos(NomosPackage),
    Logos {
        items: Vec<CoreItem>,
        names: NameTableBytes,
    },
}

impl DocumentPayload {
    pub fn kind(&self) -> DocumentKind {
        match self {
            Self::TypeSchema { .. } => DocumentKind::TypeSchema,
            Self::SignalContract(_) => DocumentKind::SignalContract,
            Self::NexusRuntime(_) => DocumentKind::NexusRuntime,
            Self::SemaStorage(_) => DocumentKind::SemaStorage,
            Self::Nomos(_) => DocumentKind::Nomos,
            Self::Logos { .. } => DocumentKind::Logos,
        }
    }

    pub fn validate(&self) -> Result<(), RootViolation> {
        match self {
            Self::SignalContract(root) => root.validate(),
            Self::NexusRuntime(root) => root.validate(),
            Self::SemaStorage(root) => root.validate(),
            Self::TypeSchema { .. } | Self::Nomos(_) | Self::Logos { .. } => Ok(()),
        }
    }

    pub fn content_hash(&self) -> Result<ContentHash, CodecError> {
        let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(self)
            .map_err(|error| CodecError::Encode(error.to_string()))?;
        Ok(ContentHash(*blake3::hash(&bytes).as_bytes()))
    }
}

#[derive(Archive, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct StoredDocument {
    pub key: DocumentKey,
    pub version: Version,
    pub hash: ContentHash,
    pub payload: DocumentPayload,
}
#[derive(Archive, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct SlotSummary {
    pub key: DocumentKey,
    pub version: Version,
    pub hash: ContentHash,
}
#[derive(Archive, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct ChangeEvent {
    pub subscription: SubscriptionIdentifier,
    pub snapshot: Snapshot,
    pub document: SlotSummary,
}

#[derive(Archive, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum Request {
    Store {
        key: DocumentKey,
        payload: DocumentPayload,
    },
    Fetch {
        key: DocumentKey,
        version: Option<Version>,
    },
    List {
        scope: FixtureScope,
        kind: Option<DocumentKind>,
    },
    HashFetch {
        hash: ContentHash,
    },
    Snapshot {
        scope: FixtureScope,
    },
    Subscribe {
        scope: FixtureScope,
        kind: Option<DocumentKind>,
    },
    AllocateIdentifiers {
        scope: FixtureScope,
        count: u32,
    },
}
#[derive(Archive, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum Reply {
    Stored(SlotSummary),
    Document(Option<StoredDocument>),
    Listed(Vec<SlotSummary>),
    Snapshotted(Snapshot),
    Subscribed {
        identifier: SubscriptionIdentifier,
        initial: Vec<SlotSummary>,
    },
    IdentifiersAllocated(IdentifierBlock),
    Rejected(Rejection),
    Event(ChangeEvent),
}
#[derive(Archive, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum Rejection {
    InvalidKind,
    InvalidDocument(RootViolation),
    NotFound,
    CountZero,
    IncompatibleWireVersion,
    Internal,
}

#[derive(Debug, thiserror::Error)]
pub enum CodecError {
    #[error("encode: {0}")]
    Encode(String),
    #[error("decode: {0}")]
    Decode(String),
    #[error("frame too large")]
    FrameTooLarge,
}

pub struct Wire;
impl Wire {
    pub fn encode_request(value: &Request) -> Result<Vec<u8>, CodecError> {
        rkyv::to_bytes::<rkyv::rancor::Error>(value)
            .map(|bytes| bytes.to_vec())
            .map_err(|error| CodecError::Encode(error.to_string()))
    }
    pub fn encode_reply(value: &Reply) -> Result<Vec<u8>, CodecError> {
        rkyv::to_bytes::<rkyv::rancor::Error>(value)
            .map(|bytes| bytes.to_vec())
            .map_err(|error| CodecError::Encode(error.to_string()))
    }
}
