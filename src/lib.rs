#![doc = include_str!("../README.md")]
#![deny(missing_docs)]
#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

pub use crate::net::place::{Place, PlaceId, PlaceMetadata, Pn};
pub use crate::net::trans::{Tn, Trans, TransId, TransMetadata};
pub use crate::net::{Arcs, NetId, Nn, PetriNet, W};
pub use crate::plugin::PetriNetPlugin;
pub use net::token::Token;

mod net;
mod plugin;

#[cfg(test)]
mod tests {}
