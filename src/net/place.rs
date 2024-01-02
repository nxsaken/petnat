//! Petri net places.

use std::any::TypeId;
use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;

use crate::net::trans::TransId;
use crate::net::NetId;

/// Reference to a place in a net.
pub struct PlaceId<Net: NetId>(TypeId, PhantomData<Net>);

impl<Net: NetId> Clone for PlaceId<Net> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<Net: NetId> Copy for PlaceId<Net> {}

impl<Net: NetId> PartialEq<Self> for PlaceId<Net> {
    fn eq(&self, other: &Self) -> bool {
        TypeId::eq(&self.0, &other.0)
    }
}

impl<Net: NetId> Eq for PlaceId<Net> {}

impl<Net: NetId> Hash for PlaceId<Net> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl<Net: NetId> Debug for PlaceId<Net> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("PlaceId")
            .field(&self.0)
            .field(&format_args!("_"))
            .finish()
    }
}

/// Place in a Petri net.
///
/// May represent different concepts depending on the context,
/// commonly used to represent some state or condition.
pub trait Place<Net: NetId>
where
    Self: Send + Sync + 'static,
{
    /// Identifier.
    fn erased() -> PlaceId<Net> {
        PlaceId(TypeId::of::<Self>(), PhantomData)
    }
}

/// Numbered [`Place`] compatible with any [`PetriNet`] for convenience.
pub enum Pn<const N: usize> {}

impl<Net: NetId, const N: usize> Place<Net> for Pn<N> {}

#[derive(Clone, Debug)]
pub(crate) struct ErasedPlace<Net: NetId> {
    pub producers: Vec<(TransId<Net>, usize)>,
    pub consumers: Vec<(TransId<Net>, usize)>,
}

#[cfg(test)]
mod tests {}
