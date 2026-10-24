//! A command-line tool's help, written beside what it describes and
//! assembled into one Markdown manual whose reader is an agent.
//!
//! Nothing here yet: the backlog builds it (docs/agents/afkconvoy.md).

// Flux's refinements of the standard library: without them a slice has no
// length Flux knows of. Both are empty outside Flux.
extern crate flux_alloc;
extern crate flux_core;

#[cfg(flux)]
mod rejected;
