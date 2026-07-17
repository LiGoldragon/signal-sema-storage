# Architecture

This pure contract crate names typed storage, lookup, version, snapshot, push-subscription, fixture-scoped identifier-allocation, and identity-authority messages for the prototype's central Sema daemon. It owns no actors, sockets, or durable state.

## The identity-authority operation (design v2, primary-56d1.11)

The central daemon is the one logical allocation authority per deployment (settled: "seat it centrally in sema"). Its mandate is the keystone's two laws — never re-mint an identity for the same declared thing, never rebind an existing identity to a different thing. This crate names the bind-or-mint operation that enforces them:

- `Request::BindIdentities { whole: SchemaWholeHandle, declarations: Vec<DeclaredIdentity> }` — mint or bind the schema-whole's universe, then per declaration mint a fresh identity (new thing) or return the existing one (same thing).
- `Reply::IdentitiesBound(BoundIdentities)` — the whole's `MintedUniverse` and one `IdentityAssignment` per declaration (its `TypeIdentity` plus a `BindOutcome` of `Minted`/`Bound`).
- Each `DeclaredIdentity` carries a `DeclaredKey` ("the same declared thing"), a `DeclaredShape` fingerprint, and an `IdentityIntent`: `MintOrBind` (idempotent by key; a shape change under the same key is a §3 version-advance keeping the id) or `Continue(TypeIdentity)` (a rename/move asserting an existing identity — the §4 declared bind-existing marker).
- Law-2 rejection `Rejection::IdentityRebindRejected { identity, bound_shape, attempted_shape }`; plus `EmptyDeclarationSet` and `IdentityNeverMinted`.

All additions are append-only on the `Request`, `Reply`, and `Rejection` enums (new variants appended after the existing ones), so the rkyv wire encoding stays backward-compatible; `tests/round_trip.rs` covers the new request, reply, and rejection.

## Revisable leans

- The prototype stores all accepted document roots in one closed `DocumentPayload`; revise when the document-kind design review settles richer roots.
- One fixture scope is explicit on every key and allocation request. No split/merge lineage is inferred.
- **Identity-authority contract leans.** `SchemaWholeHandle` is an opaque author-supplied byte handle (`whole-handle-opaque`, v2 L1): the whole carries no independently minted-and-returned constitutive id. `DeclaredKey` keys "the same declared thing" on the declared name spelling within its whole (`declared-key-is-name`, v2 L4). Revise both if an authoring surface carries explicit whole/declaration identity markers, or the psyche rules whole-identity is independently minted. The `AllocateIdentifiers`/`IdentifierBlock` fixture-scope allocation is retained as a marked compatibility path alongside the authoritative `BindIdentities` mint.
- `signal-sema` remains the universal operation-class vocabulary; this separate crate is the component contract because `signal-sema` explicitly owns no component payloads.
- **Short-header observability (signal-frame partial adoption).** Unlike the sibling engine contracts, which hand-roll a u32-length + rkyv frame and bypass `signal-frame` entirely, this contract already carries requests over `signal-frame`'s `ExchangeFrame` with the shared handshake and version negotiation. It frames them with `ShortHeader::empty()`, however, so it never populates the short-header tap-anywhere observability fields. The lean holds while no tap consumer reads them. Revise it when cross-hop tracing or routing needs the short header — populate it at the frame boundary.
