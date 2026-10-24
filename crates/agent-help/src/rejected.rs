//! Programs Flux must reject, compiled only under Flux. Each is the smallest
//! program that breaks one rule the gate counts on Flux to enforce:
//! `#[flux::should_fail]` turns Flux's acceptance of it into an error, so
//! a checker that stopped checking fails the gate instead of passing it.

#![allow(dead_code)]

// An index is checked against the length: the slice may be empty.
#[flux::should_fail]
fn first(bytes: &[u8]) -> u8 {
    bytes[0]
}

// Arithmetic may not overflow (`check_overflow = "strict"`).
#[flux::should_fail]
fn sum(a: u8, b: u8) -> u8 {
    a + b
}
