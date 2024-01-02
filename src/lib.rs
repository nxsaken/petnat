#![doc = include_str!("../README.md")]
#![deny(missing_docs)]
#![deny(clippy::all)]

pub use crate::net::place::{Place, PlaceId, Pn};
pub use crate::net::trans::{Arcs, Tn, Trans, TransId, W};
pub use crate::net::{NetId, Nn, PetriNet, PetriNetBuilder};
pub use crate::plugin::PetriNetPlugin;
pub use crate::token::Token;

mod net;
mod plugin;
mod token;

#[cfg(test)]
mod tests {}
