//! Petri net token.

use crate::net::place::PlaceId;
use bevy::prelude::Component;
use bevy::utils::HashMap;
use error::NotEnoughMarks;
use std::cmp::Ordering;

/// Petri net token. Holds the state of the net execution.
#[derive(Component, Clone, Eq, PartialEq, Default, Debug)]
pub struct Token {
    marking: HashMap<PlaceId, usize>,
}

impl Token {
    /// Returns a new token.
    pub fn new() -> Self {
        Self {
            marking: HashMap::new(),
        }
    }

    /// Returns the number of times this token has marked a place.
    pub fn marking(&self, place: PlaceId) -> usize {
        self.marking.get(&place).copied().unwrap_or(0)
    }

    /// Marks a place with this token `n` times.
    /// Returns the resulting number of markings of the place.
    pub fn mark(&mut self, place: PlaceId, n: usize) -> usize {
        *self
            .marking
            .entry(place)
            .and_modify(|m| *m += n)
            .or_insert(n)
    }

    /// Undoes `n` markings of a place by this token.
    /// Returns the resulting number of markings of the place.
    pub fn unmark(&mut self, place: PlaceId, n: usize) -> Result<usize, NotEnoughMarks> {
        let Some(marking) = self.marking.get_mut(&place) else {
            return Err(NotEnoughMarks {
                place,
                expected: n,
                actual: 0,
            });
        };
        match <usize>::cmp(marking, &n) {
            Ordering::Less => Err(NotEnoughMarks {
                place,
                expected: n,
                actual: *marking,
            }),
            Ordering::Equal => {
                self.marking.remove(&place);
                Ok(0)
            }
            Ordering::Greater => {
                *marking -= n;
                Ok(*marking)
            }
        }
    }
}

/// Token-related errors.
pub mod error {
    use crate::net::place::PlaceId;
    use bevy::utils::thiserror::Error;

    #[allow(missing_docs)]
    #[derive(Debug, Error)]
    #[error("{place:?} was expected to be marked at least {expected:?} times, actual: {actual:?}.")]
    pub struct NotEnoughMarks {
        pub place: PlaceId,
        pub expected: usize,
        pub actual: usize,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PLACE_ID: PlaceId = PlaceId(0);
    const N: usize = 3;

    #[test]
    fn test_new_token_has_no_markings() {
        let token = Token::new();
        assert!(token.marking.is_empty());
        assert_eq!(token.marking(PLACE_ID), 0);
    }

    #[test]
    fn test_marking_a_place_adds_to_token() {
        let mut token = Token::new();
        let marking_before = token.marking(PLACE_ID);
        assert_eq!(token.mark(PLACE_ID, N), marking_before + N);
    }

    #[test]
    fn test_unmarking_a_place_removes_from_token() {
        let mut token = Token::new();
        let markings_before = token.mark(PLACE_ID, N);
        let remaining_markings = token.unmark(PLACE_ID, N).unwrap();
        assert_eq!(remaining_markings, markings_before - N);
    }

    #[test]
    fn test_unmarking_more_than_marked_returns_error() {
        let mut token = Token::new();
        token.mark(PLACE_ID, N);
        let err = token.unmark(PLACE_ID, N + 1);
        assert!(err.is_err());
        let err = err.unwrap_err();
        assert_eq!(err.expected, N + 1);
        assert_eq!(err.actual, N);
    }
}
