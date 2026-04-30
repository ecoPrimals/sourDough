# {{PRIMAL_NAME}} -- Specification

**Version:** 0.1.0 (Draft)
**Status:** Architectural Specification
**Author:** ecoPrimals Project
**Date:** {{DATE}}
**License:** AGPL-3.0-or-later (scyBorg Provenance Trio)

---

## Abstract

{{PRIMAL_NAME}} is {{BRIEF_DESCRIPTION}}.

---

## 1. Core Principles

### 1.1 {{PRINCIPLE_1}}

{{DESCRIPTION}}

### 1.2 {{PRINCIPLE_2}}

{{DESCRIPTION}}

---

## 2. Data Model

### 2.1 Primary Data Structure

```rust
/// {{DESCRIPTION}}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct {{PRIMARY_STRUCT}} {
    /// Unique identifier
    pub id: {{ID_TYPE}},
}
```

### 2.2 Secondary Structures

{{ADDITIONAL_STRUCTURES}}

---

## 3. Architecture

### 3.1 Component Overview

```
+---------------------------------------------------------------+
|                     {{PRIMAL_NAME}} Service                    |
+---------------------------------------------------------------+
|                                                               |
|  +-----------+  +-----------+  +---------------------------+  |
|  | Component |  | Component |  |       Component 3         |  |
|  |     1     |  |     2     |  |                           |  |
|  +-----+-----+  +-----+-----+  +-------------+-------------+  |
|        |              |                      |                |
|        v              v                      v                |
|  +-----------------------------------------------------------+|
|  |                    Storage Layer                           ||
|  +-----------------------------------------------------------+|
|                                                               |
+---------------------------------------------------------------+
```

### 3.2 Component Details

{{COMPONENT_DESCRIPTIONS}}

---

## 4. IPC Specification

### 4.1 JSON-RPC 2.0 Methods (Primary)

```json
// Method naming: domain.verb
{
    "jsonrpc": "2.0",
    "method": "{{primal_name}}.{{verb}}",
    "params": { "key": "value" },
    "id": 1
}
```

### 4.2 Binary RPC Service (Secondary, High-Throughput)

```rust
pub trait {{PrimalName}}Rpc {
    fn health(&self) -> impl Future<Output = Result<HealthReport, String>> + Send;
    fn state(&self) -> impl Future<Output = Result<PrimalState, String>> + Send;
    // Add primal-specific methods
}
```

---

## 5. Integration Points

### 5.1 BearDog Integration (Identity)

{{BEARDOG_INTEGRATION}}

### 5.2 Songbird Integration (Discovery)

{{SONGBIRD_INTEGRATION}}

### 5.3 Other Primal Integration

{{OTHER_INTEGRATION}}

---

## 6. Storage Model

### 6.1 Primary Storage

{{STORAGE_DESCRIPTION}}

### 6.2 Recommended Backends

| Backend | Use Case |
|---------|----------|
| {{BACKEND}} | {{USE_CASE}} |

---

## 7. Performance Targets

| Metric | Target | Notes |
|--------|--------|-------|
| {{METRIC}} | {{TARGET}} | {{NOTES}} |

---

## 8. Security Considerations

### 8.1 {{SECURITY_TOPIC}}

{{DESCRIPTION}}

---

## 9. Implementation Roadmap

### Phase 1: Core Engine (X weeks)
- [ ] {{TASK_1}}
- [ ] {{TASK_2}}

### Phase 2: {{PHASE_NAME}} (X weeks)
- [ ] {{TASK_1}}
- [ ] {{TASK_2}}

---

## 10. References

- [sourDough Core](../sourDough/) -- Primal traits and scaffolding
- [BearDog](../beardog/) -- Identity and signing
- [Songbird](../songbird/) -- Service discovery
- [wateringHole Standards](../../infra/wateringHole/) -- Ecosystem standards

---

*{{PRIMAL_NAME}}: {{TAGLINE}}*
