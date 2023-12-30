#![doc = include_str!("../README.md")]
#![deny(missing_docs)]
#![deny(clippy::all)]

pub mod net;
pub mod plugin;
pub mod token;

// todo:
// - petri net resource / component
// - tokens are components, so they can be attached to entities
// - state management via petri nets (total game state & entity states)
// - safe, special cases of petri nets explored
// - workflow patterns
// - one petri net reused by multiple tokens (colored)

#[cfg(test)]
mod tests {}
