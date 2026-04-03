//! Command modules for the `SourDough` CLI.

#![expect(
    clippy::redundant_pub_crate,
    reason = "pub(crate) is explicit about intent in binary crate modules"
)]

pub(crate) mod doctor;
pub(crate) mod genomebin;
pub(crate) mod scaffold;
pub(crate) mod validate;
