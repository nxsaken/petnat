//! Petri net token.

use std::cmp::Ordering;
use std::collections::HashMap;
use std::marker::PhantomData;

use crate::net::place::{Place, PlaceId};
use crate::net::NetId;

/// Petri net token. Holds the state of the net execution.
/// TODO: make this a Bevy component
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Token<Net: NetId> {
    marking: HashMap<PlaceId<Net>, usize>,
    _net: PhantomData<Net>,
}

impl<Net: NetId> Default for Token<Net> {
    fn default() -> Self {
        Self {
            marking: HashMap::new(),
            _net: PhantomData,
        }
    }
}

impl<Net: NetId> Token<Net> {
    /// Returns a new token.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the number of times this token has marked a place.
    pub fn marks_by_id(&self, place: PlaceId<Net>) -> usize {
        self.marking.get(&place).copied().unwrap_or(0)
    }

    /// Returns the number of times this token has marked a place.
    pub fn marks<P: Place<Net>>(&self) -> usize {
        self.marks_by_id(P::ID)
    }

    /// Marks a place with this token `n` times.
    pub fn mark_by_id(&mut self, place: PlaceId<Net>, n: usize) {
        self.marking
            .entry(place)
            .and_modify(|m| *m += n)
            .or_insert(n);
    }

    /// Marks a place with this token `n` times.
    pub fn mark<P: Place<Net>>(&mut self, n: usize) {
        self.mark_by_id(P::ID, n);
    }

    /// Undoes `n` markings of a place by this token.
    pub fn unmark_by_id(&mut self, place: PlaceId<Net>, n: usize) -> Option<()> {
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

    /// Undoes `n` markings of a place by this token.
    pub fn unmark<P: Place<Net>>(&mut self, n: usize) -> Option<()> {
        self.unmark_by_id(P::ID, n)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::place::P;
    use crate::net::N;

    const N: usize = 3;

    #[test]
    fn test_new_token_has_no_markings() {
        let token = Token::<N<0>>::default();
        assert!(token.marking.is_empty());
        assert_eq!(token.marks::<P<0>>(), 0);
    }

    #[test]
    fn test_marking_a_place_adds_to_token() {
        let mut token = Token::<N<0>>::default();
        let m0 = token.marks::<P<0>>();
        token.mark::<P<0>>(N);
        let m1 = token.marks::<P<0>>();
        assert_eq!(m1, m0 + N);
    }

    #[test]
    fn test_unmarking_a_place_removes_from_token() {
        let mut token = Token::<N<0>>::default();
        token.mark::<P<0>>(N);
        let m0 = token.marks::<P<0>>();
        token.unmark::<P<0>>(N).unwrap();
        let m1 = token.marks::<P<0>>();
        assert_eq!(m1, m0 - N);
    }

    #[test]
    fn test_unmarking_more_than_marked_returns_none() {
        let mut token = Token::<N<0>>::default();
        token.mark::<P<0>>(N);
        assert!(token.unmark::<P<0>>(N + 1).is_none());
    }
}
