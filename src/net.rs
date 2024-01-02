//! Petri net.

use std::fmt::Debug;

use bevy_ecs::system::Resource;
use bevy_utils::StableHashMap;

use crate::net::place::{ErasedPlace, Place, PlaceId};
use crate::net::trans::{Arcs, ErasedTrans, Trans, TransId};
use crate::token::Token;

pub mod place;
pub mod trans;

/// Label for [`PetriNet`].
pub trait NetId: Send + Sync + 'static {}

/// Numbered [`NetId`] for convenience.
pub enum Nn<const N: usize> {}

impl<const N: usize> NetId for Nn<N> {}

/// Petri net.
///
/// TODO:
///  - special cases of PNs at the type level?
///  - deadlock detection / other useful algorithms
#[derive(Resource, Clone, Debug)]
pub struct PetriNet<Net: NetId> {
    places: StableHashMap<PlaceId<Net>, ErasedPlace<Net>>,
    transitions: StableHashMap<TransId<Net>, ErasedTrans<Net>>,
}

impl<Net: NetId> PetriNet<Net> {
    /// Creates a [`PetriNet`] builder.
    pub fn builder() -> PetriNetBuilder<Net> {
        PetriNetBuilder {
            places: StableHashMap::default(),
            transitions: StableHashMap::default(),
        }
    }

    /// Spawns new token.
    pub fn spawn_token(&self) -> Token<Net> {
        Token::new()
    }

    /// Returns whether a transition is enabled.
    pub fn enabled<T: Trans<Net>>(&self, token: &Token<Net>) -> bool {
        self.get_enabled_by_id(T::erased(), token).is_some()
    }

    /// Fires a transition.
    pub fn fire<T: Trans<Net>>(&self, token: &mut Token<Net>) -> Option<()> {
        self.fire_by_id(T::erased(), token)
    }

    /// Fires all transitions once.
    pub fn fire_all(&self, token: &mut Token<Net>) -> Vec<TransId<Net>> {
        self.transitions
            .keys()
            .copied()
            .filter(|&trans| self.fire_by_id(trans, token).is_some())
            .collect()
    }

    /// Fires transitions until the net is dead.
    ///
    /// Will loop forever if the token moves in a cycle.
    pub fn fire_while(&self, token: &mut Token<Net>) {
        loop {
            let Some(trans) = self
                .transitions
                .keys()
                .copied()
                .find(|trans| self.get_enabled_by_id(*trans, token).is_some())
            else {
                break;
            };
            self.fire_by_id(trans, token);
        }
    }

    /// Marks a place with this token `n` times.
    pub fn mark<P: Place<Net>>(&self, token: &mut Token<Net>, n: usize) {
        let id = P::erased();
        assert!(self.places.contains_key(&id), "Place {id:?} not found.");
        token.mark_by_id(id, n);
    }

    /// Undoes `n` markings of a place by this token.
    pub fn unmark<P: Place<Net>>(&self, token: &mut Token<Net>, n: usize) -> Option<()> {
        let id = P::erased();
        assert!(self.places.contains_key(&id), "Place {id:?} not found.");
        token.unmark_by_id(id, n)
    }

    /// Tries to return an enabled transition.
    fn get_enabled_by_id(
        &self,
        trans: TransId<Net>,
        token: &Token<Net>,
    ) -> Option<&ErasedTrans<Net>> {
        let Some(trans) = self.transitions.get(&trans) else {
            panic!("Transition {trans:?} not found.");
        };
        trans
            .join
            .iter()
            .all(|&(place, weight)| token.marks_by_id(place) >= weight)
            .then_some(trans)
    }

    /// Fires transition.
    fn fire_by_id(&self, trans: TransId<Net>, token: &mut Token<Net>) -> Option<()> {
        let ErasedTrans { join, split } = self.get_enabled_by_id(trans, token)?;
        join.iter()
            .map(|&(p, w)| token.unmark_by_id(p, w))
            .for_each(|um| um.expect("place unmarked"));
        split.iter().for_each(|&(p, w)| token.mark_by_id(p, w));
        Some(())
    }
}

/// [`PetriNet`] builder.
pub struct PetriNetBuilder<Net: NetId> {
    places: StableHashMap<PlaceId<Net>, ErasedPlace<Net>>,
    transitions: StableHashMap<TransId<Net>, ErasedTrans<Net>>,
}

impl<Net: NetId> PetriNetBuilder<Net> {
    /// Adds a [`Place`] to the net.
    pub fn add_place<P: Place<Net>>(mut self) -> Self {
        self.add_place_by_id(P::erased());
        self
    }

    /// Adds a [`Trans`] and its input and output [`Arcs`] to the net.
    pub fn add_trans<T: Trans<Net>, I: Arcs<Net>, O: Arcs<Net>>(mut self) -> Self {
        self.add_trans_by_id(T::erased(), I::erased(), O::erased());
        self
    }

    /// Returns the constructed [`PetriNet`].
    pub fn build(self) -> PetriNet<Net> {
        // todo: validation
        PetriNet {
            places: self.places,
            transitions: self.transitions,
        }
    }

    fn add_place_by_id(&mut self, place: PlaceId<Net>) {
        assert!(
            !self.places.contains_key(&place),
            "Attempted to add place {:?} twice.",
            place
        );
        self.places.insert(
            place,
            ErasedPlace {
                producers: vec![],
                consumers: vec![],
            },
        );
    }

    fn add_trans_by_id(
        &mut self,
        trans: TransId<Net>,
        preset: Vec<(PlaceId<Net>, usize)>,
        postset: Vec<(PlaceId<Net>, usize)>,
    ) {
        assert!(
            !self.transitions.contains_key(&trans),
            "Attempted to add transition {:?} twice.",
            trans
        );
        for (place, weight) in preset.iter() {
            let Some(place) = self.places.get_mut(place) else {
                panic!("Input place {place:?} not found.");
            };
            place.consumers.push((trans, *weight));
        }
        for (place, weight) in postset.iter() {
            let Some(place) = self.places.get_mut(place) else {
                panic!("Output place {place:?} not found.");
            };
            place.producers.push((trans, *weight));
        }
        self.transitions.insert(
            trans,
            ErasedTrans {
                join: preset,
                split: postset,
            },
        );
    }
}

#[cfg(test)]
mod tests {
    use super::place::Place;
    use super::trans::{Trans, W};
    use super::{NetId, PetriNet};

    enum Minimal {}
    enum ProdCons {}
    enum Star {}
    enum Ring {}
    enum Choice {}

    enum P0 {}
    enum P1 {}
    enum P2 {}
    enum P3 {}
    enum P4 {}

    enum T0 {}
    enum T1 {}

    impl NetId for Minimal {}
    impl NetId for ProdCons {}
    impl NetId for Star {}
    impl NetId for Ring {}
    impl NetId for Choice {}

    impl<Net: NetId> Place<Net> for P0 {}
    impl<Net: NetId> Place<Net> for P1 {}
    impl<Net: NetId> Place<Net> for P2 {}
    impl<Net: NetId> Place<Net> for P3 {}
    impl<Net: NetId> Place<Net> for P4 {}

    impl<Net: NetId> Trans<Net> for T0 {}
    impl<Net: NetId> Trans<Net> for T1 {}

    // (p0) -> |t0| -> (p1)
    fn minimal() -> PetriNet<Minimal> {
        PetriNet::builder()
            .add_place::<P0>()
            .add_place::<P1>()
            .add_trans::<T0, (P0, W<1>), (P1, W<1>)>()
            .build()
    }

    // Transitions with no input places are token sources,
    // and transitions with no output places are token sinks
    // |t0| -> (p0) -> |t1|
    fn producer_consumer() -> PetriNet<ProdCons> {
        PetriNet::builder()
            .add_place::<P0>()
            .add_trans::<T0, (), (P0, W<1>)>()
            .add_trans::<T1, (P0, W<1>), ()>()
            .build()
    }

    // (p0) -\            /-> (p2)
    //        >-> |t0| --<--> (p3)
    // (p1) -/            \-> (p4)
    fn weighted_star() -> PetriNet<Star> {
        PetriNet::builder()
            .add_place::<P0>()
            .add_place::<P1>()
            .add_place::<P2>()
            .add_place::<P3>()
            .add_place::<P4>()
            .add_trans::<T0, ((P0, W<1>), (P1, W<2>)), ((P2, W<1>), (P3, W<2>), (P4, W<3>))>()
            .build()
    }

    // Two places sending a token back and forth through two transitions in opposite directions:
    //  /--> |t0| -> (p1)
    // (p0) <- |t1| <--/
    fn ring() -> PetriNet<Ring> {
        PetriNet::builder()
            .add_place::<P0>()
            .add_place::<P1>()
            .add_trans::<T0, (P0, W<1>), (P1, W<1>)>()
            .add_trans::<T1, (P1, W<1>), (P0, W<1>)>()
            .build()
    }

    // Two transitions sharing a preset place. When one of them fires, the other ceases to be enabled.
    // (p0) --> |t0| -\
    // (p1) -<         >-> (p3)
    // (p2) --> |t1| -/
    fn choice() -> PetriNet<Choice> {
        PetriNet::builder()
            .add_place::<P0>()
            .add_place::<P1>()
            .add_place::<P2>()
            .add_place::<P3>()
            .add_trans::<T0, ((P0, W<1>), (P1, W<1>)), (P3, W<1>)>()
            .add_trans::<T1, ((P1, W<1>), (P2, W<1>)), (P3, W<1>)>()
            .build()
    }

    #[test]
    fn test_minimal() {
        let net = minimal();
        let mut token = net.spawn_token();
        net.mark::<P0>(&mut token, 1);
        assert!(net.fire::<T0>(&mut token).is_some());
        assert_eq!(token.marks::<P0>(), 0);
        assert_eq!(token.marks::<P1>(), 1);
    }

    #[test]
    fn test_producer_consumer() {
        let net = producer_consumer();
        let mut token = net.spawn_token();
        assert!(net.fire::<T0>(&mut token).is_some());
        assert_eq!(token.marks::<P0>(), 1);
        assert!(net.fire::<T1>(&mut token).is_some());
        assert_eq!(token.marks::<P0>(), 0);
    }

    #[test]
    fn test_weighted_star() {
        let net = weighted_star();
        let mut token = net.spawn_token();
        net.mark::<P0>(&mut token, 1);
        net.mark::<P1>(&mut token, 2);
        assert!(net.fire::<T0>(&mut token).is_some());
        assert_eq!(token.marks::<P0>(), 0);
        assert_eq!(token.marks::<P1>(), 0);
        assert_eq!(token.marks::<P2>(), 1);
        assert_eq!(token.marks::<P3>(), 2);
        assert_eq!(token.marks::<P4>(), 3);
    }

    #[test]
    fn test_ring() {
        let net = ring();
        let mut token = net.spawn_token();
        net.mark::<P0>(&mut token, 1);
        assert_eq!(token.marks::<P0>(), 1);
        assert_eq!(token.marks::<P1>(), 0);
        assert!(net.fire::<T0>(&mut token).is_some());
        assert_eq!(token.marks::<P0>(), 0);
        assert_eq!(token.marks::<P1>(), 1);
        assert!(net.fire::<T1>(&mut token).is_some());
        assert_eq!(token.marks::<P0>(), 1);
        assert_eq!(token.marks::<P1>(), 0);
    }

    #[test]
    fn test_choice() {
        let net = choice();
        let mut token = net.spawn_token();
        net.mark::<P0>(&mut token, 1);
        net.mark::<P1>(&mut token, 1);
        net.mark::<P2>(&mut token, 1);
        assert!(net.enabled::<T0>(&token));
        assert!(net.enabled::<T1>(&token));
        assert!(net.fire::<T0>(&mut token).is_some());
        assert!(!net.enabled::<T1>(&token));
    }
}
