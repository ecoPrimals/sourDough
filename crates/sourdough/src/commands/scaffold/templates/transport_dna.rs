//! Transport + protocol negotiation DNA templates.
//!
//! These templates give scaffolded primals G66 transport abstraction and
//! G65 protocol negotiation capabilities — self-contained, zero sourDough dep.

/// G66 Transport Abstraction module template.
pub(in crate::commands::scaffold) const TRANSPORT_RS: &str = include_str!("dna/transport.rs.tmpl");

/// G65 Protocol Negotiation module template.
pub(in crate::commands::scaffold) const PROTOCOL_NEGOTIATION_RS: &str =
    include_str!("dna/protocol_negotiation.rs.tmpl");
