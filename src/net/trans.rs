//! Petri net transitions.

use crate::net::place::{Place, PlaceId};
use crate::net::NetId;
use std::marker::PhantomData;

/// Reference to a transition.
/// TODO: replace usize with T: Hash?
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub struct TransId<Net: NetId>(pub(crate) usize, PhantomData<Net>);

impl<Net: NetId> TransId<Net> {
    /// Returns a new transition identifier.
    pub const fn new(id: usize) -> Self {
        Self(id, PhantomData)
    }
}

/// Transition in a Petri net.
///
/// TODO: derive macro
pub trait Trans<Net: NetId>
where
    Self: Send + Sync + 'static,
{
    /// Identifier.
    const ID: TransId<Net>;
}

/// Universal numbered transition for convenience.
pub enum T<const ID: usize> {}

impl<Net: NetId, const ID: usize> Trans<Net> for T<ID> {
    const ID: TransId<Net> = TransId::new(ID);
}

#[derive(Clone, Debug)]
pub(crate) struct TransNode<Net: NetId> {
    pub join: Vec<(PlaceId<Net>, usize)>,
    pub split: Vec<(PlaceId<Net>, usize)>,
}

impl<Net: NetId> TransNode<Net> {
    pub fn new(join: Vec<(PlaceId<Net>, usize)>, split: Vec<(PlaceId<Net>, usize)>) -> Self {
        Self { join, split }
    }
}

/// Arc weight.
pub trait Weight {
    /// Arc weight value.
    const W: usize;
}

/// Weight of 1.
pub struct W1;

/// Weight of `W` + 1.
pub struct Succ<W>(PhantomData<W>);

impl Weight for W1 {
    const W: usize = 1;
}

impl<W: Weight> Weight for Succ<W> {
    const W: usize = W::W + 1;
}

/// Weight of 2.
pub type W2 = Succ<W1>;

/// Weight of 3.
pub type W3 = Succ<W2>;

/// Weighted place-transition arcs.
pub trait Arcs<Net: NetId> {
    /// Returns a vector of weighted arcs.
    fn arcs() -> Vec<(PlaceId<Net>, usize)>;
}

macro_rules! impl_arcs {
    ($(($place:ident, $weight:ident)),+) => {
        #[allow(unused_parens)]
        impl<Net, $($place, $weight),+> Arcs<Net> for ($(($place, $weight)),+)
        where
            Net: NetId,
            $($place: Place<Net>, $weight: Weight),+
        {
            fn arcs() -> Vec<(PlaceId<Net>, usize)> {
                vec![$(($place::ID, $weight::W)),+]
            }
        }
    };
}

impl_arcs!((P0, W0));
impl_arcs!((P0, W0), (P1, W1));
impl_arcs!((P0, W0), (P1, W1), (P2, W2));
impl_arcs!((P0, W0), (P1, W1), (P2, W2), (P3, W3));

#[cfg(test)]
mod tests {}
