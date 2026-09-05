# sflow-api

**The embedding contract for the [SFLOW](https://structured.flow) workflow engine.**

This crate contains only stable trait contracts and wire types. No implementation, no business logic, no integrity enforcement — just the public API surface that the proprietary `sflow-engine` binary implements and that consumer applications import.

| | |
| --- | --- |
| **License** | MIT OR Apache-2.0 (your choice) |
| **MSRV** | 1.85 |
| **Engine** | proprietary, binary distribution at [structured.flow/releases](https://structured.flow) |

## What this crate gives you

```rust
use sflow_api::traits::{WorkflowBackend, WorkflowBackendExt, PersistenceBackend, ConsensusBackend};
use sflow_api::types::{StatechartDef, WorkflowInstance, WorkflowEvent, NamespaceId};
use sflow_api::error::SflowError;
```

- **`WorkflowBackend`** — minimal stable trait every workflow engine implements (start, send_event, query, signal, cancel, list)
- **`WorkflowBackendExt`** — extended capabilities (correlate, suspend, resume, migrate, search, get_state)
- **`PersistenceBackend`** — pluggable storage layer (save_instance, claim_instance, append_event, search, …)
- **`ConsensusBackend`** — pluggable consistency layer (CAS, leader election)
- **Wire types** — `StatechartDef`, `WorkflowInstance`, `EventRecord`, `WorkflowEvent`, …
- **Errors** — `SflowError` enum

The crate is **trait-only**. Add it to your `Cargo.toml`:

```toml
[dependencies]
sflow-api = "0.1"
```

## How to use it

### Option 1 — Link the official proprietary engine

Download the proprietary `libsflow-engine` binary from <https://structured.flow/releases/embedded/>, place it where `build.rs` finds it, and use the official engine through this contract.

```rust
use sflow_api::traits::WorkflowBackend;
// ...code that consumes WorkflowBackend trait, never depends on a concrete engine...

let backend: Arc<dyn WorkflowBackend> = create_official_engine(persistence)?;
backend.create(&namespace, "approval_chain", context, &owner, vec![]).await?;
```

### Option 2 — Write your own implementation behind the trait

Test mocks, alternative engines, plugin shims — all fine. The trait is the boundary; the implementation is yours.

```rust
struct MyMockBackend;

#[async_trait::async_trait]
impl WorkflowBackend for MyMockBackend {
    async fn create(/* ... */) -> Result<Uuid, SflowError> { /* ... */ }
    async fn send_event(/* ... */) -> Result<(), SflowError> { /* ... */ }
    // ...
}
```

### Option 3 — Build a third-party fork behind the same trait

Proprietary forks, OSS reimplementations, hybrid engines — all permitted. The trait carries no license restriction.

## Stability promise

`sflow-api` follows **strict SemVer**:

- **Patch** (`0.1.x`) — no API changes
- **Minor** (`0.x`) — additive changes only (new trait methods with default impls, new types, new enum variants marked `#[non_exhaustive]`)
- **Major** (`x.0`) — breaking changes; migration guide ships with each major release

Once `1.0` lands, breaking changes happen at most yearly with overlapping support window.

## Composition model

This crate is the boundary between three license worlds:

```
┌───────────────────────────────────────────────────────────┐
│  PROPRIETARY  ←──implements──┐                            │
│  (sflow-engine binary)        │                            │
│                                ▼                            │
│           sflow-api  (MIT OR Apache-2.0, this crate)       │
│                                ▲                            │
│  CONSUMER  ←──depends──────────┘                            │
│  (any license: Apache, MIT, AGPL, BSD, proprietary, …)     │
└───────────────────────────────────────────────────────────┘
```

Anyone can write code on either side without contaminating the other. AGPL applications (like CoordiNode CE) import `sflow-api` without inheriting AGPL; proprietary applications link the engine without inheriting any open-source obligation.

## Related crates

- `sflow-proto` — gRPC service definitions (Apache-2.0, public on GitHub)
- `@structured-world/sflow-vue-ce` — Vue 3 components for workflow UIs (Apache-2.0, npm)

## Links

- [Engine downloads](https://structured.flow/releases)
- [Embedding guide](https://structured.flow/docs/embedding)
- [Statechart format reference (xstate v5 compatible)](https://structured.flow/docs/statechart)
