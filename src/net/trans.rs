//! Petri net transitions.

use bevy_utils::StableHashMap;
use educe::Educe;
use std::any::{type_name, TypeId};
use std::borrow::Cow;
use std::marker::PhantomData;

use crate::net::place::{Place, PlaceId, PlaceMetadata};
use crate::net::NetId;

/// Transition belonging to a Petri net.
pub trait Trans<Net: NetId>: Send + Sync + 'static {}

/// Numbered [`Trans`] compatible with any Petri net for convenience.
pub enum Tn<const N: usize> {}

impl<Net: NetId, const N: usize> Trans<Net> for Tn<N> {}

/// Reference to a [`Trans`] in a Petri net.
#[derive(Educe)]
#[educe(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default)]
pub struct TransId<Net: NetId>(usize, PhantomData<Net>);

impl<Net: NetId> TransId<Net> {
    /// Creates a new [`TransId`].
    ///
    /// The `index` is a unique value associated with each type of transition in a given Petri net.
    /// This value is taken from a counter incremented for each type of transition registered with the Petri net.
    const fn new(index: usize) -> Self {
        Self(index, PhantomData)
    }

    /// Returns the index of the transition.
    #[inline]
    #[must_use]
    pub const fn index(self) -> usize {
        self.0
    }
}

/// Transition metadata.
#[derive(Educe)]
#[educe(Debug, Default)]
pub struct TransMetadata<Net: NetId> {
    name: Cow<'static, str>,
    type_id: Option<TypeId>,
    _net: PhantomData<Net>,
}

impl<Net: NetId> TransMetadata<Net> {
    /// Create a new [`TransInfo`] for the transition `T`.
    #[must_use]
    pub fn new<T: Trans<Net>>() -> Self {
        Self {
            name: Cow::Borrowed(type_name::<T>()),
            type_id: Some(TypeId::of::<T>()),
            _net: PhantomData,
        }
    }

    /// Returns the name of the transition.
    #[inline]
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the [`TypeId`] of the transition.
    ///
    /// ## Panics
    ///
    /// Panics if the transition does not correspond to a Rust type.
    #[inline]
    #[must_use]
    pub fn type_id(&self) -> TypeId {
        self.type_id.expect("type_id present")
    }

    /// Returns the [`TypeId`] of the transition.
    ///
    /// Returns `None` if the transition does not correspond to a Rust type.
    #[inline]
    #[must_use]
    pub const fn get_type_id(&self) -> Option<TypeId> {
        self.type_id
    }
}

#[derive(Educe)]
#[educe(Debug, Default)]
pub struct Transitions<Net: NetId> {
    transitions: Vec<TransMetadata<Net>>,
    indices: StableHashMap<TypeId, TransId<Net>>,
}

impl<Net: NetId> Transitions<Net> {
    /// Registers a transition of type `T` with the Petri net.
    ///
    /// The returned `TransId` is specific to the Petri net instance
    /// it was retrieved from and should not be used with another Petri net.
    ///
    /// ## Panics
    ///
    /// Panics if a transition of this type has already been initialized.
    #[inline]
    pub fn register<T: Trans<Net>>(&mut self) -> TransId<Net> {
        let Transitions {
            transitions,
            indices,
        } = self;
        *indices
            .try_insert(
                TypeId::of::<T>(),
                Self::init_inner(transitions, TransMetadata::new::<T>()),
            )
            .unwrap_or_else(|_| {
                panic!(
                    "Attempted to add a duplicate transition: {}",
                    type_name::<T>()
                )
            })
    }

    /// Registers a transition via its metadata.
    ///
    /// The returned `TransId` is specific to the Petri net instance
    /// it was retrieved from and should not be used with another Petri net.
    ///
    /// ## Note
    ///
    /// If this method is called multiple times with identical metadata,
    /// a distinct [`TransId`] will be created for each one.
    pub fn _register_with_info(&mut self, meta: TransMetadata<Net>) -> TransId<Net> {
        Self::init_inner(&mut self.transitions, meta)
    }

    #[inline]
    fn init_inner(
        transitions: &mut Vec<TransMetadata<Net>>,
        meta: TransMetadata<Net>,
    ) -> TransId<Net> {
        let index = TransId::new(transitions.len());
        transitions.push(meta);
        index
    }

    /// Returns the number of transitions registered with the Petri net.
    #[inline]
    pub fn _len(&self) -> usize {
        self.transitions.len()
    }

    /// Returns `true` iff there are no transitions registered with the Petri net.
    #[inline]
    pub fn _is_empty(&self) -> bool {
        self.transitions.is_empty()
    }

    /// Returns the metadata associated with the given transition.
    #[inline]
    pub fn _metadata(&self, id: TransId<Net>) -> &TransMetadata<Net> {
        self.transitions.get(id.index()).unwrap_or_else(|| {
            panic!(
                "Transition `{:?}` not found in net `{}`. Make sure you register it first.",
                id,
                type_name::<Net>()
            )
        })
    }

    /// Returns the name associated with the given transition.
    #[inline]
    pub fn _name(&self, id: TransId<Net>) -> &str {
        self._metadata(id).name()
    }

    /// Returns the [`TransId`] associated with the `type_id`.
    ///
    /// The returned `TransId` is specific to the Petri net instance
    /// it was retrieved from and should not be used with another Petri net.
    ///
    /// ## Panics
    ///
    /// Panics if the `type_id` has not been registered with the Petri net.
    #[inline]
    pub fn _id_from_erased(&self, type_id: TypeId) -> TransId<Net> {
        self.indices.get(&type_id).copied().unwrap_or_else(|| {
            panic!(
                "Transition `{:?}` not found in net `{}`. Make sure you register it first.",
                type_id,
                type_name::<Net>()
            )
        })
    }

    /// Returns the [`TransId`] associated with the type `T`.
    ///
    /// The returned `TransId` is specific to the Petri net instance
    /// it was retrieved from and should not be used with another Petri net.
    ///
    /// ## Panics
    ///
    /// Panics if the `Trans` type has not been registered with the Petri net.
    #[inline]
    pub fn id<T: Trans<Net>>(&self) -> TransId<Net> {
        self.indices
            .get(&TypeId::of::<T>())
            .copied()
            .unwrap_or_else(|| {
                panic!(
                    "Transition `{}` not found in net `{}`. Make sure you register it first.",
                    type_name::<T>(),
                    type_name::<Net>()
                )
            })
    }

    /// Gets an iterator over all transition metadata registered with the Petri net.
    #[inline]
    pub fn _iter(&self) -> impl Iterator<Item = &TransMetadata<Net>> + '_ {
        self.transitions.iter()
    }
}

#[derive(Educe)]
#[educe(Debug, Default)]
pub(crate) struct Inflow<Net: NetId> {
    pub source: PlaceId<Net>,
    pub weight: usize,
}

#[derive(Educe)]
#[educe(Debug, Default)]
pub(crate) struct Outflow<Net: NetId> {
    pub target: PlaceId<Net>,
    pub weight: usize,
}

#[derive(Educe)]
#[educe(Debug, Default)]
pub(crate) struct Flows<Net: NetId> {
    inflows: Vec<Vec<Inflow<Net>>>,
    outflows: Vec<Vec<Outflow<Net>>>,
}

impl<Net: NetId> Flows<Net> {
    pub fn add_inflows(&mut self, inflows: Vec<Inflow<Net>>) {
        self.inflows.push(inflows);
    }

    pub fn add_outflows(&mut self, outflows: Vec<Outflow<Net>>) {
        self.outflows.push(outflows);
    }

    pub fn inflows(&self, trans: TransId<Net>) -> &[Inflow<Net>] {
        &self.inflows[trans.index()]
    }

    pub fn outflows(&self, trans: TransId<Net>) -> &[Outflow<Net>] {
        &self.outflows[trans.index()]
    }
}

/// Arc weight.
pub enum W<const N: usize> {}

/// Weighted place-transition arcs.
pub trait Arcs<Net: NetId> {
    /// Returns a vector of erased arcs.
    fn erased() -> Vec<(PlaceMetadata<Net>, usize)>;
}

// single place case
impl<Net, P0, const W0: usize> Arcs<Net> for (P0, W<W0>)
where
    Net: NetId,
    P0: Place<Net>,
{
    fn erased() -> Vec<(PlaceMetadata<Net>, usize)> {
        vec![(PlaceMetadata::new::<P0>(), W0)]
    }
}

macro_rules! impl_arcs {
    ($(($place:ident, $weight:ident)),*) => {
        #[allow(unused_parens)]
        impl<Net, $($place, const $weight: usize),*> Arcs<Net> for ($(($place, W<$weight>),)*)
        where
            Net: NetId,
            $($place: Place<Net>),*
        {
            fn erased() -> Vec<(PlaceMetadata<Net>, usize)> {
                vec![$((PlaceMetadata::new::<$place>(), $weight)),*]
            }
        }
    };
}

impl_arcs!();
impl_arcs!((P0, W0));
impl_arcs!((P0, W0), (P1, W1));
impl_arcs!((P0, W0), (P1, W1), (P2, W2));
impl_arcs!((P0, W0), (P1, W1), (P2, W2), (P3, W3));

#[cfg(test)]
mod tests {}
