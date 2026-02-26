//! Generated protobuf types for the IB platform wire protocol.
//!
//! These types are the canonical cross-language contract.  Use them for
//! NATS binary encoding instead of ad-hoc JSON when performance matters.

pub mod v1 {
    include!(concat!(env!("OUT_DIR"), "/ib.platform.v1.rs"));
}
