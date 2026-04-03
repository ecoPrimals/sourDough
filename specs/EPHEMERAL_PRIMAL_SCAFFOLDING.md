# Ephemeral Primal Scaffolding

**Date:** April 3, 2026
**Version:** 0.2.0 (proposed for v0.7+)
**Status:** Specification Draft
**License:** AGPL-3.0-or-later
**Origin:** ludoSpring V17 — session-as-primal composition pattern

---

## Motivation

sourDough's `PrimalLifecycle` trait defines a state machine for long-lived
primals. But some entities are naturally **short-lived**: a game session
that lasts 20 minutes, an NPC that exists for a quest, a multiplayer match
that dies when the last player leaves, a content mod loaded at runtime.

These entities benefit from primal-level infrastructure (IPC surface,
health checks, capability advertising, provenance, identity) but their
lifetime is bounded by a parent process. They are **ephemeral primals**.

This specification extends sourDough's trait patterns to support ephemeral
primal scaffolding — enabling any process to spawn short-lived primals
with full lifecycle management.

---

## Lifecycle

Ephemeral primals reuse `PrimalLifecycle`'s state machine unchanged:

```
Created → Starting → Running → Stopping → Stopped
                                              ↘ Failed
```

The difference is operational, not structural:

| Aspect | Long-lived primal | Ephemeral primal |
|--------|-------------------|------------------|
| Spawned by | biomeOS deploy graph | Parent process at runtime |
| Lifetime | Hours to permanent | Seconds to hours |
| Registration | Graph startup | `lifecycle.register` on spawn |
| Teardown | Graph shutdown | `lifecycle.deregister` + `stop()` |
| Provenance | Persists with primal | **Outlives runtime** |
| Discovery | Static capability set | Scoped capabilities (namespaced) |

### Spawn Protocol

```
1. Parent creates the ephemeral struct (PrimalLifecycle::state() == Created)
2. Parent calls start() (transitions to Starting → Running)
3. If biomeOS is running:
   a. Call lifecycle.register with ephemeral's capabilities and ephemeral=true
   b. biomeOS acknowledges and routes capability queries to the ephemeral
4. If rhizoCrypt is running:
   a. Create a session DAG for the ephemeral (dag.session.create)
   b. Record spawn event as first vertex
```

### Teardown Protocol

```
1. Parent decides ephemeral is done (session end, quest complete, match over)
2. If rhizoCrypt is running:
   a. Record teardown event as final vertex
   b. DAG remains queryable after teardown
3. If biomeOS is running:
   a. Call lifecycle.deregister with ephemeral's primal_id
   b. biomeOS confirms: capability queries no longer route to the dead ephemeral
4. Parent calls stop() (transitions to Stopping → Stopped)
5. Parent drops the ephemeral struct
```

### Provenance Outlives Runtime

The key insight: an ephemeral primal's rhizoCrypt DAG persists after the
primal dies. A game session's history survives its runtime. This means:

- Replay validation works on dead sessions
- Achievement certificates reference DAG vertices from expired primals
- Audit trails span multiple ephemeral lifetimes
- Post-mortem analysis doesn't require the primal to be running

---

## Parent Management

Ephemeral primals always have a **parent** that owns their lifetime:

```rust
struct EphemeralOwner<T: PrimalLifecycle> {
    ephemeral: T,
    primal_id: String,
    registered: bool,
}

impl<T: PrimalLifecycle> EphemeralOwner<T> {
    fn spawn(ephemeral: T, primal_id: String) -> Result<Self, PrimalError> {
        let mut owner = Self { ephemeral, primal_id, registered: false };
        owner.ephemeral.start()?;
        // Register with biomeOS if available
        owner.try_register();
        Ok(owner)
    }

    fn teardown(mut self) -> Result<(), PrimalError> {
        self.try_deregister();
        self.ephemeral.stop()
    }
}

impl<T: PrimalLifecycle> Drop for EphemeralOwner<T> {
    fn drop(&mut self) {
        // Safety net: deregister on drop if still registered
        if self.registered {
            self.try_deregister();
        }
    }
}
```

The parent is responsible for:

1. Creating the ephemeral with appropriate configuration
2. Starting the lifecycle
3. Registering with biomeOS (if available)
4. Monitoring health during the ephemeral's lifetime
5. Deciding when to tear down
6. Ensuring deregistration even on panic (Drop guard)

---

## Capability Surface

Ephemeral primals advertise capabilities **scoped by instance**:

```
session.{id}.game.tick
session.{id}.game.state
session.{id}.game.complete
```

The namespace prevents collision when multiple ephemerals of the same type
run concurrently (e.g., multiple game sessions on the same host).

biomeOS routes capability queries by prefix matching:
- `capability.resolve("session.42.game.tick")` → routes to session 42
- `capability.resolve("session.*.game.tick")` → routes to any session

---

## Health Contract

Ephemeral primals implement `PrimalHealth`:

| State | `health.liveness` response |
|-------|---------------------------|
| Created | `Unhealthy { reason: "not started" }` |
| Starting | `Degraded { reason: "starting" }` |
| Running | `Healthy` |
| Stopping | `Degraded { reason: "stopping" }` |
| Stopped | `Unhealthy { reason: "stopped" }` |
| Failed | `Unhealthy { reason: "..." }` |

After teardown, the ephemeral no longer responds to health probes (it is
deregistered from biomeOS). Any cached health status should be interpreted
as stale.

---

## Use Cases

### Session-as-Primal

A game session is scaffolded as an ephemeral primal. ludoSpring's game
science runs through the session's IPC surface. biomeOS manages the
lifecycle. rhizoCrypt records every tick as a DAG vertex. When the player
quits, the session primal dies, but the DAG persists for replay validation
and achievement certification.

**Capabilities:** `session.{id}.game.tick`, `session.{id}.game.state`,
`session.{id}.game.complete`

### NPC-as-Primal

Complex NPCs get their own primal identity. Each NPC has an independent
Squirrel (AI) context, its own rhizoCrypt state, and a capability surface
for dialogue. biomeOS coordinates a village of NPC primals into an emergent
narrative mesh. When the NPC despawns, its history persists for narrative
continuity.

**Capabilities:** `npc.{id}.dialogue`, `npc.{id}.state`, `npc.{id}.action`

### Mod-as-Primal

A content pack is scaffolded as a primal. biomeOS discovers mods by
capability, not by filename. loamSpine certifies the mod. sweetGrass
attributes the creator. When the mod is unloaded, its attribution chain
persists.

**Capabilities:** `mod.{id}.content.*`, `mod.{id}.rules.*`

### Match-as-Primal

A multiplayer game server is a primal spawned per match. Songbird
discovers it for matchmaking. Players connect. When the match ends,
the primal dies, the session DAG is dehydrated to loamSpine as a
certified match result.

**Capabilities:** `match.{id}.join`, `match.{id}.state`, `match.{id}.leave`

---

## Implementation Guidance

### Minimum Viable Ephemeral

The simplest ephemeral primal needs:

1. A struct with `PrimalState` tracking
2. `start()` / `stop()` methods following `PrimalLifecycle` semantics
3. A capability list scoped by instance ID
4. A health method returning status based on state

It does **not** need:

- Its own binary (runs in-process with the parent)
- Its own socket (routes through parent or biomeOS)
- Persistent configuration (ephemeral by definition)

### Integration with biomeOS

When biomeOS is available, the ephemeral registers via:

```json
{
    "jsonrpc": "2.0",
    "method": "lifecycle.register",
    "params": {
        "primal_id": "session.42.exp074",
        "capabilities": ["session.42.game.tick", "session.42.game.state"],
        "ephemeral": true,
        "parent": "ludospring"
    },
    "id": 1
}
```

The `ephemeral: true` flag tells biomeOS this primal has a bounded lifetime
and should be cleaned up from routing tables on deregistration or timeout.

### Integration with rhizoCrypt

Each ephemeral gets its own DAG session:

```json
{
    "jsonrpc": "2.0",
    "method": "dag.session.create",
    "params": {
        "name": "session.42",
        "ephemeral": true,
        "parent_session": "ludospring_main"
    },
    "id": 1
}
```

The `parent_session` link enables DAG hierarchy: a ludoSpring main session
contains sub-DAGs for each ephemeral game session.

---

## Evolution Path

This specification is a draft. Expected evolution:

1. **v0.7+**: `EphemeralOwner<T>` utility in `sourdough-core`
2. **v0.8+**: biomeOS native ephemeral support (timeout, auto-cleanup)
3. **v0.9+**: Ephemeral-to-permanent promotion (an NPC that becomes important
   enough gets promoted to a long-lived primal)
4. **v1.0+**: Ephemeral mesh coordination (swarm of ephemeral primals
   coordinated by biomeOS as a single logical entity)

---

## References

- `sourDough/specs/ARCHITECTURE.md` — PrimalLifecycle, PrimalHealth, PrimalDiscovery traits
- `sourDough/specs/SOURDOUGH_SPECIFICATION.md` — sourDough's role as starter culture
- `ludoSpring/specs/PRIMAL_LEVERAGE_MAP.md` — game science leverage map
- `ludoSpring/experiments/exp074_session_as_primal/` — reference implementation
- `wateringHole/PRIMAL_SPRING_GARDEN_TAXONOMY.md` — ecosystem layers
