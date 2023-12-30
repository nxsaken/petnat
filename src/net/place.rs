//! Petri net places.

use crate::net::NetId;
use std::marker::PhantomData;

/// Reference to a place.
/// TODO: replace usize with T: Hash? – or with TypeId for better type safety
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub struct PlaceId<Net: NetId>(pub(crate) usize, PhantomData<Net>);

impl<Net: NetId> PlaceId<Net> {
    /// Returns a new place identifier.
    pub const fn new(id: usize) -> Self {
        Self(id, PhantomData)
    }
}

/// Place in a Petri net.
///
/// May represent different concepts depending on the context,
/// commonly used to represent some state or condition.
///
/// TODO: derive macro
pub trait Place<Net: NetId>
where
    Self: Send + Sync + 'static,
{
    /// Identifier.
    const ID: PlaceId<Net>;
}

/// Universal numbered place for convenience.
pub enum P<const ID: usize> {}

impl<Net: NetId, const ID: usize> Place<Net> for P<ID> {
    const ID: PlaceId<Net> = PlaceId::new(ID);
}

#[cfg(test)]
mod tests {}
