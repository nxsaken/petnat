//! Petri net places.

/// Reference to a place. todo: maybe use a Handle?
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub struct PlaceId(pub(crate) usize);

/// Place in a Petri net.
///
/// May represent different concepts depending on the context,
/// commonly used to represent some state or condition.
#[derive(Clone, Eq, PartialEq, Hash, Default, Debug)]
pub struct Place;

impl Place {
    /// Returns a new place.
    pub const fn new() -> Self {
        Self
    }
}

#[cfg(test)]
mod tests {}
