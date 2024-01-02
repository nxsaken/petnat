//! Petri net token.

use std::cmp::Ordering;
use std::collections::HashMap;
use std::marker::PhantomData;

use bevy_ecs::component::Component;

use crate::net::place::{Place, PlaceId};
use crate::net::NetId;

/// Petri net token. Holds the state of the net execution.
///
/// TODO: WorldQuery for querying tokens with a specific marking
#[derive(Component, Clone, Eq, PartialEq, Debug)]
pub struct Token<Net: NetId> {
    marking: HashMap<PlaceId<Net>, usize>,
    _net: PhantomData<Net>,
}

impl<Net: NetId> Token<Net> {
    /// Returns a new token.
    pub(crate) fn new() -> Self {
        Self {
            marking: HashMap::new(),
            _net: PhantomData,
        }
    }

    /// Returns the number of times a token has marked a place.
    pub fn marks<P: Place<Net>>(&self) -> usize {
        self.marks_by_id(P::erased())
    }

    /// Returns the total number of markings of the token.
    pub fn total_marks(&self) -> usize {
        self.marking.values().sum()
    }

    pub(crate) fn marks_by_id(&self, place: PlaceId<Net>) -> usize {
        self.marking.get(&place).copied().unwrap_or(0)
    }

    pub(crate) fn mark_by_id(&mut self, place: PlaceId<Net>, n: usize) {
        self.marking
            .entry(place)
            .and_modify(|m| *m += n)
            .or_insert(n);
    }

    pub(crate) fn unmark_by_id(&mut self, place: PlaceId<Net>, n: usize) -> Option<()> {
        let Some(marking) = self.marking.get_mut(&place) else {
            return None;
        };
        match <usize>::cmp(marking, &n) {
            Ordering::Less => None,
            Ordering::Equal => {
                self.marking.remove(&place);
                Some(())
            }
            Ordering::Greater => {
                *marking -= n;
                Some(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::net::trans::{Trans, W};
    use crate::net::PetriNet;

    use super::*;

    enum N0 {}
    enum P0 {}
    enum T0 {}

    impl NetId for N0 {}
    impl Place<N0> for P0 {}
    impl Trans<N0> for T0 {}

    const N: usize = 3;

    fn net() -> PetriNet<N0> {
        PetriNet::builder()
            .add_place::<P0>()
            .add_trans::<T0, (P0, W<1>), ()>()
            .build()
    }

    #[test]
    fn test_new_token_has_no_markings() {
        let net = net();
        let token = net.spawn_token();
        assert_eq!(token.total_marks(), 0);
    }

    #[test]
    fn test_marking_a_place_adds_to_token() {
        let net = net();
        let mut token = net.spawn_token();
        let m0 = token.marks::<P0>();
        net.mark::<P0>(&mut token, N);
        let m1 = token.marks::<P0>();
        assert_eq!(m1, m0 + N);
    }

    #[test]
    fn test_unmarking_a_place_removes_from_token() {
        let net = net();
        let mut token = net.spawn_token();
        net.mark::<P0>(&mut token, N);
        let m0 = token.marks::<P0>();
        net.unmark::<P0>(&mut token, N).unwrap();
        let m1 = token.marks::<P0>();
        assert_eq!(m1, m0 - N);
    }

    #[test]
    fn test_unmarking_more_than_marked_returns_none() {
        let net = net();
        let mut token = net.spawn_token();
        net.mark::<P0>(&mut token, N);
        assert!(net.unmark::<P0>(&mut token, N + 1).is_none());
    }
}
