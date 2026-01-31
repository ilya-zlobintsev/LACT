#![allow(
    non_upper_case_globals,
    non_camel_case_types,
    non_snake_case,
    unused,
    unknown_lints,
    unnecessary_transmutes,
    unsafe_op_in_unsafe_fn,
    clippy::too_many_arguments,
    clippy::pedantic,
    clippy::upper_case_acronyms,
    clippy::ptr_offset_with_cast
)]

pub mod intel {
    include!(concat!(env!("OUT_DIR"), "/intel_bindings.rs"));
}

#[cfg(feature = "nvidia")]
pub mod nvidia {
    include!(concat!(env!("OUT_DIR"), "/nvidia_bindings.rs"));
}
