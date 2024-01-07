//! Petri net places.

use bevy_utils::StableHashMap;
use educe::Educe;
use std::any::{type_name, TypeId};
use std::borrow::Cow;
use std::marker::PhantomData;

use crate::net::NetId;

/// Place belonging to a Petri net.
///
/// May represent different concepts depending on the context,
/// commonly used to represent some state or condition.
pub trait Place<Net: NetId>: Send + Sync + 'static {}

/// Numbered [`Place`] compatible with any Petri net for convenience.
pub enum Pn<const N: usize> {}

impl<Net: NetId, const N: usize> Place<Net> for Pn<N> {}

/// Reference to a [`Place`] in a Petri net.
#[derive(Educe)]
#[educe(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default)]
pub struct PlaceId<Net: NetId>(usize, PhantomData<Net>);

impl<Net: NetId> PlaceId<Net> {
    /// Creates a new [`PlaceId`].
    ///
    /// The `index` is a unique value associated with each type of place in a given Petri net.
    /// Usually, this value is taken from a counter incremented for each type of place registered with the Petri net.
    #[inline]
    #[must_use]
    const fn new(index: usize) -> Self {
        Self(index, PhantomData)
    }

    /// Returns the index of the current place.
    #[inline]
    #[must_use]
    pub const fn index(self) -> usize {
        self.0
    }
}

/// A value describing a [`Place`], which may or may not correspond to a Rust type.
#[derive(Educe)]
#[educe(Clone, Debug, Default)]
pub struct PlaceMetadata<Net: NetId> {
    name: Cow<'static, str>,
    type_id: Option<TypeId>,
    _net: PhantomData<Net>,
}

impl<Net: NetId> PlaceMetadata<Net> {
    /// Create a new [`PlaceMetadata`] for the place `P`.
    #[must_use]
    pub fn new<P: Place<Net>>() -> Self {
        Self {
            name: Cow::Borrowed(type_name::<P>()),
            type_id: Some(TypeId::of::<P>()),
            _net: PhantomData,
        }
    }

    /// Returns the name of the place.
    #[inline]
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the [`TypeId`] of the place.
    ///
    /// ## Panics
    ///
    /// Panics if the place does not correspond to a Rust type.
    #[inline]
    #[must_use]
    pub fn type_id(&self) -> TypeId {
        self.type_id.expect("type_id present")
    }

    /// Returns the [`TypeId`] of the place.
    ///
    /// Returns `None` if the place does not correspond to a Rust type.
    #[inline]
    #[must_use]
    pub const fn get_type_id(&self) -> Option<TypeId> {
        self.type_id
    }
}

#[derive(Educe)]
#[educe(Debug, Default)]
pub struct Places<Net: NetId> {
    places: Vec<PlaceMetadata<Net>>,
    indices: StableHashMap<TypeId, PlaceId<Net>>,
}

impl<Net: NetId> Places<Net> {
    /// Registers a place of type `P` with this instance.
    ///
    /// The returned `PlaceId` is specific to the Petri net instance
    /// it was retrieved from and should not be used with another Petri net.
    ///
    /// ## Panics
    ///
    /// Panics if a place of this type has already been initialized.
    #[inline]
    pub fn register<P: Place<Net>>(&mut self) -> PlaceId<Net> {
        let Places { places, indices } = self;
        *indices
            .try_insert(
                TypeId::of::<P>(),
                Self::init_inner(places, PlaceMetadata::new::<P>()),
            )
            .unwrap_or_else(|_| panic!("Attempted to add a duplicate place: {}", type_name::<P>()))
    }

    /// Registers a place via its metadata.
    ///
    /// The returned `PlaceId` is specific to the Petri net instance
    /// it was retrieved from and should not be used with another Petri net.
    ///
    /// ## Note
    ///
    /// If this method is called multiple times with identical metadata,
    /// a distinct [`PlaceId`] will be created for each one.
    pub fn _register_with_info(&mut self, meta: PlaceMetadata<Net>) -> PlaceId<Net> {
        Self::init_inner(&mut self.places, meta)
    }

    #[inline]
    fn init_inner(places: &mut Vec<PlaceMetadata<Net>>, meta: PlaceMetadata<Net>) -> PlaceId<Net> {
        let index = PlaceId::new(places.len());
        places.push(meta);
        index
    }

    /// Returns the number of places registered with this instance.
    #[inline]
    pub fn len(&self) -> usize {
        self.places.len()
    }

    /// Returns `true` if there are no places registered with this instance. Otherwise, this returns `false`.
    #[inline]
    pub fn _is_empty(&self) -> bool {
        self.places.is_empty()
    }

    /// Gets the metadata associated with the given place.
    #[inline]
    pub fn _metadata(&self, id: PlaceId<Net>) -> &PlaceMetadata<Net> {
        &self.places[id.index()]
    }

    /// Returns the name associated with the given place.
    ///
    /// This will return an incorrect result if `id` did not come from the same Petri net as `self`.
    /// It may return `None` or a garbage value.
    #[inline]
    pub fn _name(&self, id: PlaceId<Net>) -> &str {
        self._metadata(id).name()
    }

    /// Returns the [`PlaceId`] associated with the given `type_id`.
    ///
    /// The returned `PlaceId` is specific to the Petri net instance
    /// it was retrieved from and should not be used with another Petri net.
    ///
    /// ## Panics
    ///
    /// Panics if the `type_id` has not been registered with the Petri net.
    #[inline]
    pub fn id_from_erased(&self, type_id: TypeId) -> PlaceId<Net> {
        self.indices.get(&type_id).copied().unwrap_or_else(|| {
            panic!(
                "Place with `{:?}` not found in net `{}`. Make sure you register it first.",
                type_id,
                type_name::<Net>()
            )
        })
    }

    /// Returns the [`PlaceId`] of the given [`Place`] of type `P`.
    ///
    /// The returned `PlaceId` is specific to the Petri net instance
    /// it was retrieved from and should not be used with another Petri net.
    ///
    /// ## Panics
    ///
    /// Panics if the `Place` type has not been registered with the Petri net.
    #[inline]
    pub fn id<P: Place<Net>>(&self) -> PlaceId<Net> {
        self.indices
            .get(&TypeId::of::<P>())
            .copied()
            .unwrap_or_else(|| {
                panic!(
                    "Place `{}` not found in net `{}`. Make sure you register it first.",
                    type_name::<P>(),
                    type_name::<Net>()
                )
            })
    }

    /// Gets an iterator over all places registered with this instance.
    #[inline]
    pub fn _iter(&self) -> impl Iterator<Item = &PlaceMetadata<Net>> + '_ {
        self.places.iter()
    }
}

#[cfg(test)]
mod tests {}
