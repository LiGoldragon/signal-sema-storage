# Architecture

This pure contract crate names typed storage, lookup, version, snapshot, push-subscription, and fixture-scoped identifier-allocation messages for the prototype's central Sema daemon. It owns no actors, sockets, or durable state.

## Revisable leans

- The prototype stores all accepted document roots in one closed `DocumentPayload`; revise when the document-kind design review settles richer roots.
- One fixture scope is explicit on every key and allocation request. No split/merge lineage is inferred.
- `signal-sema` remains the universal operation-class vocabulary; this separate crate is the component contract because `signal-sema` explicitly owns no component payloads.
- **Short-header observability (signal-frame partial adoption).** Unlike the sibling engine contracts, which hand-roll a u32-length + rkyv frame and bypass `signal-frame` entirely, this contract already carries requests over `signal-frame`'s `ExchangeFrame` with the shared handshake and version negotiation. It frames them with `ShortHeader::empty()`, however, so it never populates the short-header tap-anywhere observability fields. The lean holds while no tap consumer reads them. Revise it when cross-hop tracing or routing needs the short header — populate it at the frame boundary.
