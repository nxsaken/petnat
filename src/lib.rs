#![doc = include_str!("../README.md")]
#![deny(missing_docs)]
#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

pub use crate::net::place::{Place, PlaceId, PlaceMetadata, Pn};
pub use crate::net::trans::{Arcs, Tn, Trans, TransId, TransMetadata, W};
pub use crate::net::{NetId, Nn, PetriNet};
pub use crate::plugin::PetriNetPlugin;
pub use crate::token::Token;

mod net;
mod plugin;
mod token;

#[cfg(test)]
mod tests {}
