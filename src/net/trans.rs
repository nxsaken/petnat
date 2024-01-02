//! Petri net transitions.

use std::any::TypeId;
use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;

use crate::net::place::{Place, PlaceId};
use crate::net::NetId;

/// Reference to a transition in a net.
pub struct TransId<Net: NetId>(TypeId, PhantomData<Net>);

impl<Net: NetId> Clone for TransId<Net> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<Net: NetId> Copy for TransId<Net> {}

impl<Net: NetId> PartialEq<Self> for TransId<Net> {
    fn eq(&self, other: &Self) -> bool {
        TypeId::eq(&self.0, &other.0)
    }
}

impl<Net: NetId> Eq for TransId<Net> {}

impl<Net: NetId> Hash for TransId<Net> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl<Net: NetId> Debug for TransId<Net> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("TransId")
            .field(&self.0)
            .field(&format_args!("_"))
            .finish()
    }
}

/// Transition in a Petri net.
pub trait Trans<Net: NetId>
where
    Self: Send + Sync + 'static,
{
    /// Identifier.
    fn erased() -> TransId<Net> {
        TransId(TypeId::of::<Self>(), PhantomData)
    }
}

/// Numbered [`Trans`] compatible with any [`PetriNet`] for convenience.
pub enum Tn<const N: usize> {}

impl<Net: NetId, const N: usize> Trans<Net> for Tn<N> {}

#[derive(Clone, Debug)]
pub(crate) struct ErasedTrans<Net: NetId> {
    pub join: Vec<(PlaceId<Net>, usize)>,
    pub split: Vec<(PlaceId<Net>, usize)>,
}

/// Arc weight.
pub enum W<const N: usize> {}

/// Weighted place-transition arcs.
pub trait Arcs<Net: NetId> {
    /// Returns a vector of erased arcs.
    fn erased() -> Vec<(PlaceId<Net>, usize)>;
}

macro_rules! impl_arcs {
    ($(($place:ident, $weight:ident)),*) => {
        #[allow(unused_parens)]
        impl<Net, $($place, const $weight: usize),*> Arcs<Net> for ($(($place, W<$weight>)),*)
        where
            Net: NetId,
            $($place: Place<Net>),*
        {
            fn erased() -> Vec<(PlaceId<Net>, usize)> {
                vec![$(($place::erased(), $weight)),*]
            }
        }
    };
}

impl_arcs!();
impl_arcs!((P0, W0));

// 1-tuple case
impl<Net, P0, const W0: usize> Arcs<Net> for ((P0, W<W0>),)
where
    Net: NetId,
    P0: Place<Net>,
{
    fn erased() -> Vec<(PlaceId<Net>, usize)> {
        vec![(P0::erased(), W0)]
    }
}

impl_arcs!((P0, W0), (P1, W1));
impl_arcs!((P0, W0), (P1, W1), (P2, W2));
impl_arcs!((P0, W0), (P1, W1), (P2, W2), (P3, W3));

#[cfg(test)]
mod tests {}
