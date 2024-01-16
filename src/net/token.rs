//! Petri net token.

use std::marker::PhantomData;

use bevy_ecs::component::Component;
use educe::Educe;

use super::place::PlaceId;
use super::{NetId, NotEnoughMarks};

/// Petri net token. Holds the state of the net execution.
///
// TODO: WorldQuery for querying tokens with a specific marking
#[derive(Component, Educe)]
#[educe(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Token<Net: NetId> {
    marking: Vec<usize>,
    _net: PhantomData<Net>,
}

impl<Net: NetId> Token<Net> {
    /// Returns a new token.
    pub(super) fn new(num_places: usize) -> Self {
        Self {
            marking: vec![0; num_places],
            _net: PhantomData,
        }
    }

    /// Returns the total number of markings by a token.
    #[inline]
    #[must_use]
    pub fn total_marks(&self) -> usize {
        self.marking.iter().sum()
    }

    pub(super) fn marks_by_id(&self, place: PlaceId<Net>) -> usize {
        self.marking[place.index()]
    }

    pub(super) fn mark_by_id(&mut self, place: PlaceId<Net>, n: usize) {
        self.marking[place.index()] += n;
    }

    pub(super) fn unmark_by_id(
        &mut self,
        place: PlaceId<Net>,
        n: usize,
    ) -> Result<(), NotEnoughMarks<Net>> {
        if self.marking[place.index()] >= n {
            self.marking[place.index()] -= n;
            Ok(())
        } else {
            Err(NotEnoughMarks(place))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{NetId, PetriNet, Place, Trans, W};

    enum N0 {}
    enum P0 {}
    enum T0 {}

    impl NetId for N0 {}
    impl Place<N0> for P0 {}
    impl Trans<N0> for T0 {}

    const N: usize = 3;

    fn net() -> PetriNet<N0> {
        PetriNet::new()
            .add_place::<P0>()
            .add_trans::<T0, (P0, W<1>), ()>()
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
        let m0 = net.marks::<P0>(&token);
        net.mark::<P0>(&mut token, N);
        let m1 = net.marks::<P0>(&token);
        assert_eq!(m1, m0 + N);
    }

    #[test]
    fn test_unmarking_a_place_removes_from_token() {
        let net = net();
        let mut token = net.spawn_token();
        net.mark::<P0>(&mut token, N);
        let m0 = net.marks::<P0>(&token);
        net.unmark::<P0>(&mut token, N).unwrap();
        let m1 = net.marks::<P0>(&token);
        assert_eq!(m1, m0 - N);
    }

    #[test]
    fn test_unmarking_more_than_marked_fails() {
        let net = net();
        let mut token = net.spawn_token();
        net.mark::<P0>(&mut token, N);
        assert!(net.unmark::<P0>(&mut token, N + 1).is_err());
    }
}
