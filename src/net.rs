//! Petri net.

use bevy_ecs::system::Resource;
use bevy_utils::thiserror::Error;
use educe::Educe;

use crate::net::place::{Place, Places};
use crate::net::trans::{Arcs, Flows, Inflow, Outflow, Trans, TransId, Transitions};
use crate::token::Token;
use crate::PlaceId;

pub mod place;
pub mod trans;

/// Label for a Petri net.
pub trait NetId: Send + Sync + 'static {}

/// Numbered [`NetId`] for convenience.
pub enum Nn<const N: usize> {}

impl<const N: usize> NetId for Nn<N> {}

/// Error signifying that the transition was not enabled.
#[derive(Error, Educe)]
#[educe(Debug)]
#[error("Transition {0:?} is not enabled.")]
pub struct NotEnabled<Net: NetId>(pub TransId<Net>);

/// Error signifying that the place did not have enough tokens to be unmarked.
#[derive(Error, Educe)]
#[educe(Debug)]
#[error("Place {0:?} does not have enough marks.")]
pub struct NotEnoughMarks<Net: NetId>(pub PlaceId<Net>);

/// Petri net.
///
/// TODO:
///  - special cases of PNs at the type level?
///  - deadlock detection / other useful algorithms
#[derive(Resource, Educe)]
#[educe(Debug, Default)]
pub struct PetriNet<Net: NetId> {
    places: Places<Net>,
    transitions: Transitions<Net>,
    flows: Flows<Net>,
}

impl<Net: NetId> PetriNet<Net> {
    /// Returns an empty Petri net.
    #[must_use]
    pub fn new() -> Self {
        Self {
            places: Places::default(),
            transitions: Transitions::default(),
            flows: Flows::default(),
        }
    }

    /// Spawns new token.
    #[must_use]
    pub fn spawn_token(&self) -> Token<Net> {
        Token::new(self.places.len())
    }

    /// Returns the number of times a place has been marked by a token.
    #[must_use]
    pub fn marks<P: Place<Net>>(&self, token: &Token<Net>) -> usize {
        token.marks_by_id(self.places.id::<P>())
    }

    /// Returns whether a transition is enabled.
    #[must_use]
    pub fn enabled<T: Trans<Net>>(&self, token: &Token<Net>) -> bool {
        let trans = self.transitions.id::<T>();
        self.enabled_by_id(trans, token)
    }

    /// Fires a transition.
    ///
    /// ## Errors
    ///
    /// Returns [`NotEnabled`] if the transition is not enabled.
    pub fn fire<T: Trans<Net>>(&self, token: &mut Token<Net>) -> Result<(), NotEnabled<Net>> {
        let trans = self.transitions.id::<T>();
        self.fire_by_id(trans, token)
    }

    /// Marks a place with this token `n` times.
    pub fn mark<P: Place<Net>>(&self, token: &mut Token<Net>, n: usize) {
        let place = self.places.id::<P>();
        token.mark_by_id(place, n);
    }

    /// Undoes `n` markings of a place by this token.
    ///
    /// ## Errors
    ///
    /// Returns [`NotEnoughMarks`] if the place does not have enough tokens to be unmarked.
    pub fn unmark<P: Place<Net>>(
        &self,
        token: &mut Token<Net>,
        n: usize,
    ) -> Result<(), NotEnoughMarks<Net>> {
        let place = self.places.id::<P>();
        token.unmark_by_id(place, n)
    }

    /// Tries to return an enabled transition.
    fn enabled_by_id(&self, trans: TransId<Net>, token: &Token<Net>) -> bool {
        self.flows
            .inflows(trans)
            .iter()
            .all(|&Inflow { source, weight }| token.marks_by_id(source) >= weight)
    }

    /// Fires transition.
    fn fire_by_id(
        &self,
        trans: TransId<Net>,
        token: &mut Token<Net>,
    ) -> Result<(), NotEnabled<Net>> {
        if !self.enabled_by_id(trans, token) {
            return Err(NotEnabled(trans));
        }
        self.flows
            .inflows(trans)
            .iter()
            .map(|&Inflow { source, weight }| token.unmark_by_id(source, weight))
            .for_each(|um| um.expect("place unmarked"));
        self.flows
            .outflows(trans)
            .iter()
            .for_each(|&Outflow { target, weight }| token.mark_by_id(target, weight));
        Ok(())
    }
}

impl<Net: NetId> PetriNet<Net> {
    /// Adds a [`Place`] to the net.
    #[must_use]
    pub fn add_place<P: Place<Net>>(mut self) -> Self {
        self.places.register::<P>();
        self
    }

    /// Adds a [`Trans`] and its input and output [`Arcs`] to the net.
    ///
    /// ## Panics
    ///
    ///
    #[must_use]
    pub fn add_trans<T: Trans<Net>, I: Arcs<Net>, O: Arcs<Net>>(mut self) -> Self {
        self.transitions.register::<T>();
        self.flows.add_inflows(
            I::erased()
                .into_iter()
                .map(|(source, weight)| Inflow {
                    source: self.places.id_from_erased(source.type_id()),
                    weight,
                })
                .collect(),
        );
        self.flows.add_outflows(
            O::erased()
                .into_iter()
                .map(|(target, weight)| Outflow {
                    target: self.places.id_from_erased(target.type_id()),
                    weight,
                })
                .collect(),
        );
        self
    }

    /// Allows composing Petri net configuration.
    #[must_use]
    pub fn compose(self, f: impl FnOnce(Self) -> Self) -> Self {
        f(self)
    }
}

#[cfg(test)]
mod tests {
    use crate::{NetId, PetriNet, Place, Trans, W};

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
        PetriNet::new()
            .add_place::<P0>()
            .add_place::<P1>()
            .add_trans::<T0, (P0, W<1>), (P1, W<1>)>()
    }

    // Transitions with no input places are token sources,
    // and transitions with no output places are token sinks
    // |t0| -> (p0) -> |t1|
    fn producer_consumer() -> PetriNet<ProdCons> {
        PetriNet::new()
            .add_place::<P0>()
            .add_trans::<T0, (), (P0, W<1>)>()
            .add_trans::<T1, (P0, W<1>), ()>()
    }

    // (p0) -\            /-> (p2)
    //        >-> |t0| --<--> (p3)
    // (p1) -/            \-> (p4)
    fn weighted_star() -> PetriNet<Star> {
        PetriNet::new()
            .add_place::<P0>()
            .add_place::<P1>()
            .add_place::<P2>()
            .add_place::<P3>()
            .add_place::<P4>()
            .add_trans::<T0, ((P0, W<1>), (P1, W<2>)), ((P2, W<1>), (P3, W<2>), (P4, W<3>))>()
    }

    // Two places sending a token back and forth through two transitions in opposite directions:
    //  /--> |t0| -> (p1)
    // (p0) <- |t1| <--/
    fn ring() -> PetriNet<Ring> {
        PetriNet::new()
            .add_place::<P0>()
            .add_place::<P1>()
            .add_trans::<T0, (P0, W<1>), (P1, W<1>)>()
            .add_trans::<T1, (P1, W<1>), (P0, W<1>)>()
    }

    // Two transitions sharing a preset place. When one of them fires, the other ceases to be enabled.
    // (p0) --> |t0| -\
    // (p1) -<         >-> (p3)
    // (p2) --> |t1| -/
    fn choice() -> PetriNet<Choice> {
        PetriNet::new()
            .add_place::<P0>()
            .add_place::<P1>()
            .add_place::<P2>()
            .add_place::<P3>()
            .add_trans::<T0, ((P0, W<1>), (P1, W<1>)), (P3, W<1>)>()
            .add_trans::<T1, ((P1, W<1>), (P2, W<1>)), (P3, W<1>)>()
    }

    #[test]
    fn test_minimal() {
        let net = minimal();
        let mut token = net.spawn_token();
        net.mark::<P0>(&mut token, 1);
        assert!(net.fire::<T0>(&mut token).is_ok());
        assert_eq!(net.marks::<P0>(&token), 0);
        assert_eq!(net.marks::<P1>(&token), 1);
    }

    #[test]
    fn test_producer_consumer() {
        let net = producer_consumer();
        let mut token = net.spawn_token();
        assert!(net.fire::<T0>(&mut token).is_ok());
        assert_eq!(net.marks::<P0>(&token), 1);
        assert!(net.fire::<T1>(&mut token).is_ok());
        assert_eq!(net.marks::<P0>(&token), 0);
    }

    #[test]
    fn test_weighted_star() {
        let net = weighted_star();
        let mut token = net.spawn_token();
        net.mark::<P0>(&mut token, 1);
        net.mark::<P1>(&mut token, 2);
        assert!(net.fire::<T0>(&mut token).is_ok());
        assert_eq!(net.marks::<P0>(&token), 0);
        assert_eq!(net.marks::<P1>(&token), 0);
        assert_eq!(net.marks::<P2>(&token), 1);
        assert_eq!(net.marks::<P3>(&token), 2);
        assert_eq!(net.marks::<P4>(&token), 3);
    }

    #[test]
    fn test_ring() {
        let net = ring();
        let mut token = net.spawn_token();
        net.mark::<P0>(&mut token, 1);
        assert_eq!(net.marks::<P0>(&token), 1);
        assert_eq!(net.marks::<P1>(&token), 0);
        assert!(net.fire::<T0>(&mut token).is_ok());
        assert_eq!(net.marks::<P0>(&token), 0);
        assert_eq!(net.marks::<P1>(&token), 1);
        assert!(net.fire::<T1>(&mut token).is_ok());
        assert_eq!(net.marks::<P0>(&token), 1);
        assert_eq!(net.marks::<P1>(&token), 0);
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
        assert!(net.fire::<T0>(&mut token).is_ok());
        assert!(!net.enabled::<T1>(&token));
    }
}
