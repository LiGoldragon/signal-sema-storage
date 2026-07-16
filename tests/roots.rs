use name_table::Identifier;
use signal_sema_storage::{
    BelongsRelation, FamilyDeclaration, NameTableBytes, NexusActorDeclaration, NexusRoute,
    NexusRuntimeRoot, OpensRelation, RootViolation, SemaStorageRoot, SignalContractRoot,
    StreamDeclaration,
};

#[test]
fn signal_contract_owns_stream_opens_and_belongs_laws() {
    let contract = Identifier::new(0);
    let stream = Identifier::new(1);
    let operation = Identifier::new(2);
    let root = SignalContractRoot {
        contract,
        streams: vec![StreamDeclaration { stream }],
        opens: vec![OpensRelation { operation, stream }],
        belongs: vec![BelongsRelation { stream, contract }],
        names: NameTableBytes(Vec::new()),
    };
    assert_eq!(root.validate(), Ok(()));
}

#[test]
fn nexus_routes_only_between_declared_actors() {
    let actor = Identifier::new(1);
    let missing = Identifier::new(2);
    let root = NexusRuntimeRoot {
        actors: vec![NexusActorDeclaration { actor }],
        routes: vec![NexusRoute {
            sender: actor,
            receiver: missing,
        }],
        names: NameTableBytes(Vec::new()),
    };
    assert_eq!(root.validate(), Err(RootViolation::UnknownRouteActor));
}

#[test]
fn sema_alone_owns_versioned_families() {
    let root = SemaStorageRoot {
        families: vec![FamilyDeclaration {
            family: Identifier::new(1),
            layout_version: 0,
        }],
        names: NameTableBytes(Vec::new()),
    };
    assert_eq!(root.validate(), Err(RootViolation::ZeroLayoutVersion));
}
