# Architecture

This pure contract crate names typed storage, lookup, version, snapshot, push-subscription, and fixture-scoped identifier-allocation messages for the prototype's central Sema daemon. It owns no actors, sockets, or durable state.

## Revisable leans

- The prototype stores all accepted document roots in one closed `DocumentPayload`; revise when the document-kind design review settles richer roots.
- One fixture scope is explicit on every key and allocation request. No split/merge lineage is inferred.
- `signal-sema` remains the universal operation-class vocabulary; this separate crate is the component contract because `signal-sema` explicitly owns no component payloads.
