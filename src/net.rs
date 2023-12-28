//! Petri net.

use crate::token::Token;
use bevy::utils::thiserror::Error;
use bevy::utils::HashMap;
use place::{Place, PlaceId};
use trans::{Gate, Trans, TransId};

pub mod place;
pub mod trans;

/// Reference to a Petri net.
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub struct NetId(usize);

/// Petri net.
#[derive(Clone, Eq, PartialEq, Default, Debug)]
pub struct Net {
    places: HashMap<PlaceId, Place>,
    transitions: HashMap<TransId, Trans>,
}

/// Enum representing a Petri net error.
#[derive(Error, Copy, Clone, PartialEq, Eq, Debug)]
pub enum NetError {
    /// Expected place was not found in the net.
    #[error("Place not found: {0:?}.")]
    PlaceNotFound(PlaceId),
    /// Expected transition was not found in the net.
    #[error("Transition not found: {0:?}.")]
    TransNotFound(TransId),
    /// Task was expected to be enabled, but it was not.
    #[error("Transition not enabled: {0:?}.")]
    NotEnabled(TransId),
}

impl Net {
    /// Adds a place and returns a reference to it.
    pub fn add_place(&mut self, place: Place) -> PlaceId {
        let new_id = PlaceId(self.places.len());
        self.places.insert(new_id, place);
        new_id
    }

    /// Adds a transition and returns a reference to it.
    pub fn add_trans(&mut self, join: Gate, split: Gate) -> TransId {
        let new_id = TransId(self.transitions.len());
        self.transitions.insert(new_id, Trans::new(join, split));
        new_id
    }

    /// Adds places to the preset of a transition.
    pub fn add_inflows(
        &mut self,
        trans: TransId,
        places: &[(PlaceId, usize)],
    ) -> Result<(), NetError> {
        self.add_flows(trans, |trans| &mut trans.join, places)
    }

    /// Adds places to the postset of a transition.
    pub fn add_outflows(
        &mut self,
        trans: TransId,
        places: &[(PlaceId, usize)],
    ) -> Result<(), NetError> {
        self.add_flows(trans, |trans| &mut trans.split, places)
    }

    fn add_flows(
        &mut self,
        trans: TransId,
        gate: fn(&mut Trans) -> &mut (Gate, HashMap<PlaceId, usize>),
        places: &[(PlaceId, usize)],
    ) -> Result<(), NetError> {
        if let Some(trans) = self.transitions.get_mut(&trans) {
            if let Some(&missing_place) = places
                .iter()
                .find_map(|(place, _)| (!self.places.contains_key(place)).then_some(place))
            {
                return Err(NetError::PlaceNotFound(missing_place));
            }
            gate(trans).1.extend(places.iter());
            return Ok(());
        }
        Err(NetError::TransNotFound(trans))
    }
}

impl Net {
    /// Tries to return an enabled transition.
    pub fn get_enabled(&self, trans: TransId, token: &Token) -> Result<&Trans, NetError> {
        let Some(maybe_enabled) = self.transitions.get(&trans) else {
            return Err(NetError::TransNotFound(trans));
        };
        let (join, preset) = &maybe_enabled.join;
        let f = |(&place, &weight)| token.marking(place) >= weight;
        let enabled = match join {
            Gate::And => preset.iter().all(f),
            Gate::Xor => preset.iter().any(f),
            Gate::Or => unimplemented!(),
        };
        match enabled {
            true => Ok(maybe_enabled),
            false => Err(NetError::NotEnabled(trans)),
        }
    }

    /// Returns an iterator over the enabled transitions for a token. FIXME: extract into struct
    pub fn iter_enabled<'a>(&'a self, token: &'a Token) -> impl Iterator<Item = TransId> + 'a {
        self.transitions
            .keys()
            .copied()
            .filter(|trans| self.get_enabled(*trans, token).is_ok())
    }

    /// Fires transition.
    ///
    /// Returns `(consumed, produced)` number of tokens
    pub fn fire(&self, trans: TransId, token: &mut Token) -> Result<(usize, usize), NetError> {
        let trans = self.get_enabled(trans, token)?;
        let (join, preset) = &trans.join;
        let consumed = match join {
            Gate::And => preset
                .iter()
                .inspect(|(&place, &weight)| {
                    token
                        .unmark(place, weight)
                        .expect("place has enough tokens");
                })
                .map(|(_, &weight)| weight)
                .sum(),
            Gate::Xor => preset
                .iter()
                .filter(|(&place, &weight)| token.unmark(place, weight).is_ok())
                .map(|(_, &weight)| weight)
                .next()
                .expect("one of the places has enough tokens"),
            Gate::Or => unimplemented!(),
        };
        let (split, postset) = &trans.split;
        let produced = match split {
            Gate::And => postset
                .iter()
                .inspect(|(&place, &weight)| {
                    token.mark(place, weight);
                })
                .map(|(_, &weight)| weight)
                .sum(),
            Gate::Xor => unimplemented!(),
            Gate::Or => unimplemented!(),
        };
        Ok((consumed, produced))
    }
}

#[cfg(test)]
mod tests {
    use super::place::{Place, PlaceId};
    use super::trans::{Gate, TransId};
    use super::Net;
    use crate::token::Token;

    // Simplest functional net:
    // (p0) -> |t0| -> (p1)
    fn minimal_net() -> (Net, [PlaceId; 2], [TransId; 1]) {
        let mut net = Net::default();
        let p = [0, 1].map(|_| net.add_place(Place::new()));
        let t = [0].map(|_| net.add_trans(Gate::And, Gate::And));
        net.add_inflows(t[0], &[(p[0], 1)]).unwrap();
        net.add_outflows(t[0], &[(p[1], 1)]).unwrap();
        (net, p, t)
    }

    // Weighted AND-gate.
    // (p0) -\     &&     /-> (p2)
    //        >-> |t0| --<--> (p3)
    // (p1) -/            \-> (p4)
    fn star_net() -> (Net, [PlaceId; 5], [TransId; 1]) {
        let mut net = Net::default();
        let p = [0, 1, 2, 3, 4].map(|_| net.add_place(Place::new()));
        let t = [0].map(|_| net.add_trans(Gate::And, Gate::And));
        net.add_inflows(t[0], &[(p[0], 1), (p[1], 2)]).unwrap();
        net.add_outflows(t[0], &[(p[2], 1), (p[3], 2), (p[4], 3)])
            .unwrap();
        (net, p, t)
    }

    // Two places sending a token back and forth through two transitions in the opposite directions:
    //  /--> |t0| -> (p1)
    // (p0) <- |t1| <--/
    fn loop_net() -> (Net, [PlaceId; 2], [TransId; 2]) {
        let mut net = Net::default();
        let p = [0, 1].map(|_| net.add_place(Place::new()));
        let t = [0, 1].map(|_| net.add_trans(Gate::And, Gate::And));
        net.add_inflows(t[0], &[(p[0], 1)]).unwrap();
        net.add_outflows(t[0], &[(p[1], 1)]).unwrap();
        net.add_inflows(t[1], &[(p[1], 1)]).unwrap();
        net.add_outflows(t[1], &[(p[0], 1)]).unwrap();
        (net, p, t)
    }

    // Two AND-gates sharing a preset place. When one of them fires, the other ceases to be enabled.
    //           &&
    // (p0) --> |t0| -\
    // (p1) -<   &&    >-> (p3)
    // (p2) --> |t1| -/
    fn implicit_choice() -> (Net, [PlaceId; 4], [TransId; 2]) {
        let mut net = Net::default();
        let p = [0, 1, 2, 3].map(|_| net.add_place(Place::new()));
        let t = [0, 1].map(|_| net.add_trans(Gate::And, Gate::And));
        net.add_inflows(t[0], &[(p[0], 1), (p[1], 1)]).unwrap();
        net.add_inflows(t[1], &[(p[1], 1), (p[2], 1)]).unwrap();
        net.add_outflows(t[0], &[(p[3], 1)]).unwrap();
        net.add_outflows(t[1], &[(p[3], 1)]).unwrap();
        (net, p, t)
    }

    #[test]
    fn test_simple_trans() {
        let (net, p, t) = minimal_net();
        let mut token = Token::default();
        assert_eq!(token.mark(p[0], 1), 1);
        assert_eq!(net.fire(t[0], &mut token).ok(), Some((1, 1)));
        assert_eq!(token.marking(p[0]), 0);
        assert_eq!(token.marking(p[1]), 1);
    }

    #[test]
    fn test_and_join_split() {
        let (net, p, t) = star_net();
        let mut token = Token::default();
        assert_eq!(token.mark(p[0], 1), 1);
        assert_eq!(token.mark(p[1], 2), 2);
        assert_eq!(net.fire(t[0], &mut token).ok(), Some((3, 6)));
        assert_eq!(token.marking(p[0]), 0);
        assert_eq!(token.marking(p[1]), 0);
        assert_eq!(token.marking(p[2]), 1);
        assert_eq!(token.marking(p[3]), 2);
        assert_eq!(token.marking(p[4]), 3);
    }

    #[test]
    fn test_loop() {
        let (net, p, t) = loop_net();
        let mut token = Token::default();
        assert_eq!(token.mark(p[0], 1), 1);
        assert_eq!(token.marking(p[0]), 1);
        assert_eq!(token.marking(p[1]), 0);
        assert_eq!(net.fire(t[0], &mut token).ok(), Some((1, 1)));
        assert_eq!(token.marking(p[0]), 0);
        assert_eq!(token.marking(p[1]), 1);
        assert_eq!(net.fire(t[1], &mut token).ok(), Some((1, 1)));
        assert_eq!(token.marking(p[0]), 1);
        assert_eq!(token.marking(p[1]), 0);
    }

    #[test]
    fn test_implicit_choice() {
        let (net, p, t) = implicit_choice();
        let mut token = Token::default();
        assert_eq!(token.mark(p[0], 1), 1);
        assert_eq!(token.mark(p[1], 1), 1);
        assert_eq!(token.mark(p[2], 1), 1);
        assert!(net.get_enabled(t[0], &token).is_ok());
        assert!(net.get_enabled(t[1], &token).is_ok());
        assert_eq!(net.fire(t[0], &mut token).ok(), Some((2, 1)));
        assert!(net.get_enabled(t[1], &token).is_err());
    }
}
