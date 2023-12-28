//! Petri net transitions.

use crate::net::place::PlaceId;
use bevy::utils::HashMap;

/// Reference to a transition.
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub struct TransId(pub(crate) usize);

/// Petri net transition.
///
/// Consumes tokens from the `join`, produces tokens into the `split`.
#[derive(Clone, Eq, PartialEq, Default, Debug)]
pub struct Trans {
    /// Joining gate and preset.
    pub(crate) join: (Gate, HashMap<PlaceId, usize>),
    /// Splitting gate and postset.
    pub(crate) split: (Gate, HashMap<PlaceId, usize>),
}

/// Transition enablement / token consumption and production rules.
#[derive(Copy, Clone, Eq, PartialEq, Default, Debug)]
pub enum Gate {
    /// Waits for / consumes all preset tokens (AND-join),
    /// or marks all postset places (AND-split).
    #[default]
    And,
    /// Waits for / consumes one preset token (XOR-join),
    /// or marks exactly one postset places (XOR-split).
    Xor,
    /// Waits for / consumes all preset tokens that can arrive (OR-join),
    /// or marks one or more postset places (OR-split).
    ///
    /// OR-join has non-local semantics, which means the whole net
    /// must be analyzed to determine whether it is enabled.
    Or,
}

impl Trans {
    /// Returns a new transition.
    pub fn new(join: Gate, split: Gate) -> Self {
        Self {
            join: (join, HashMap::new()),
            split: (split, HashMap::new()),
        }
    }
}

#[cfg(test)]
mod tests {}
