//! Generated protobuf types plus thin (de)serialize helpers.

// The generated file is included verbatim.  Inner `#![...]` attributes in the
// generated source are stripped by build.rs for edition 2024 compatibility, so
// we apply the equivalent suppressions here as outer attributes on the module.
#[allow(
    dead_code,
    missing_docs,
    non_camel_case_types,
    non_snake_case,
    non_upper_case_globals,
    trivial_casts,
    unused_mut,
    unused_results,
    unknown_lints,
    clippy::all
)]
mod generated {
    include!(concat!(env!("OUT_DIR"), "/definition.rs"));
}

pub use generated::{CarrierEntry, CarrierMap};

use crate::error::Error;
use protobuf::Message;

pub fn parse(bytes: &[u8]) -> Result<CarrierMap, Error> {
    Ok(CarrierMap::parse_from_bytes(bytes)?)
}

pub fn serialize(map: &CarrierMap) -> Result<Vec<u8>, Error> {
    Ok(map.write_to_bytes()?)
}
