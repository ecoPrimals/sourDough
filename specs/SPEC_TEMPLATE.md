# {{PRIMAL_NAME}} — Specification

**Version:** 0.1.0 (Draft)  
**Status:** Architectural Specification  
**Author:** ecoPrimals Project  
**Date:** {{DATE}}  
**License:** AGPL-3.0  

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
    
    // Add fields
}
```

### 2.2 Secondary Structures

{{ADDITIONAL_STRUCTURES}}

---

## 3. Architecture

### 3.1 Component Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                     {{PRIMAL_NAME}} Service                     │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐ │
│  │ Component 1 │  │ Component 2 │  │      Component 3        │ │
│  └──────┬──────┘  └──────┬──────┘  └───────────┬─────────────┘ │
│         │                │                      │               │
│         ▼                ▼                      ▼               │
│  ┌───────────────────────────────────────────────────────────┐ │
│  │                    Storage Layer                           │ │
│  └───────────────────────────────────────────────────────────┘ │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### 3.2 Component Details

{{COMPONENT_DESCRIPTIONS}}

---

## 4. API Specification

### 4.1 gRPC Service

```protobuf
syntax = "proto3";

package {{primal_name}}.v1;

service {{PrimalName}} {
    // Define RPCs
}
```

### 4.2 REST API

```yaml
openapi: 3.0.0
info:
  title: {{PrimalName}} API
  version: 1.0.0

paths:
  /resource:
    get:
      summary: Get resource
```

---

## 5. Integration Points

### 5.1 BearDog Integration

{{BEARDOG_INTEGRATION}}

### 5.2 Songbird Integration

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

- [SourDough Core](../sourDough/) — Common traits
- [BearDog](../beardog/) — Identity and signing
- [Songbird](../songbird/) — Service discovery

---

*{{PRIMAL_NAME}}: {{TAGLINE}}*

