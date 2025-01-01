#![allow(
    non_upper_case_globals,
    non_camel_case_types,
    non_snake_case,
    unused,
    clippy::upper_case_acronyms,
    clippy::unreadable_literal
)]

pub mod i915 {
    include!(concat!(env!("OUT_DIR"), "/i915_bindings.rs"));
}

pub mod xe {
    include!(concat!(env!("OUT_DIR"), "/xe_bindings.rs"));
}
