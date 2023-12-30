//! Petri net.

use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::hash::Hash;
use std::marker::PhantomData;

use place::{Place, PlaceId};
use trans::{Arcs, Trans, TransId, TransNode};

use crate::token::Token;

pub mod place;
pub mod trans;

/// Petri net id.
///
/// TODO: derive macro
pub trait NetId
where
    Self: Send + Sync + 'static,
    Self: Copy + Eq + Hash + Debug,
{
}

/// Numbered net label for convenience.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum N<const ID: usize> {}

impl<const ID: usize> NetId for N<ID> {}

/// Petri net.
/// TODO: remove HashSet and HashMap in favor of an array + a PHF
///  since the net will probably always be constructed at compile time?
/// TODO: make this a Bevy resource
#[derive(Clone, Debug)]
pub struct PetriNet<Net: NetId> {
    places: HashSet<PlaceId<Net>>,
    transitions: HashMap<TransId<Net>, TransNode<Net>>,
    _id: PhantomData<Net>,
}

impl<Net: NetId> PetriNet<Net> {
    /// Returns an empty net.
    pub fn empty() -> Self {
        Self {
            places: Default::default(),
            transitions: Default::default(),
            _id: PhantomData,
        }
    }

    /// Adds a place to the net.
    pub fn add_place<P: Place<Net>>(mut self) -> Self {
        self.add_place_by_id(P::ID);
        self
    }

    fn add_place_by_id(&mut self, place: PlaceId<Net>) {
        let new = self.places.insert(place);
        assert!(new, "Attempted to add place {:?} twice.", place);
    }

    /// Adds a transition and returns a reference to it.
    pub fn add_trans<T: Trans<Net>, Pre: Arcs<Net>, Post: Arcs<Net>>(mut self) -> Self {
        let preset = Pre::arcs();
        let postset = Post::arcs();
        self.add_trans_by_id(T::ID, preset, postset);
        self
    }

    fn add_trans_by_id(
        &mut self,
        trans: TransId<Net>,
        preset: Vec<(PlaceId<Net>, usize)>,
        postset: Vec<(PlaceId<Net>, usize)>,
    ) {
        let dup = self
            .transitions
            .insert(trans, TransNode::new(preset, postset));
        assert!(
            dup.is_none(),
            "Attempted to add transition {:?} twice.",
            trans
        )
    }
}

impl<Net: NetId> PetriNet<Net> {
    /// Spawns new token.
    pub fn spawn_token(&self) -> Token<Net> {
        Token::new()
    }

    /// Tries to return an enabled transition.
    pub(crate) fn get_enabled_by_id(
        &self,
        trans: TransId<Net>,
        token: &Token<Net>,
    ) -> Option<&TransNode<Net>> {
        let Some(trans) = self.transitions.get(&trans) else {
            panic!("Transition {trans:?} not found.");
        };
        trans
            .join
            .iter()
            .all(|&(place, weight)| token.marks_by_id(place) >= weight)
            .then_some(trans)
    }

    /// Returns whether a transition is enabled.
    pub fn is_enabled<T: Trans<Net>>(&self, token: &Token<Net>) -> bool {
        self.get_enabled_by_id(T::ID, token).is_some()
    }

    /// Returns an iterator over the enabled transitions for a token. FIXME: extract into struct
    pub fn iter_enabled<'a>(
        &'a self,
        token: &'a Token<Net>,
    ) -> impl Iterator<Item = TransId<Net>> + 'a {
        self.transitions
            .keys()
            .copied()
            .filter(|trans| self.get_enabled_by_id(*trans, token).is_some())
    }

    /// Fires transition.
    pub fn fire_by_id(&self, trans: TransId<Net>, token: &mut Token<Net>) -> Option<()> {
        let TransNode { join, split } = self.get_enabled_by_id(trans, token)?;
        join.iter()
            .map(|&(p, w)| token.unmark_by_id(p, w))
            .for_each(|um| um.expect("place has enough tokens"));
        split.iter().for_each(|&(p, w)| token.mark_by_id(p, w));
        Some(())
    }

    /// Fires transition.
    pub fn fire<T: Trans<Net>>(&self, token: &mut Token<Net>) -> Option<()> {
        self.fire_by_id(T::ID, token)
    }
}

#[cfg(test)]
mod tests {
    use super::place::{Place, PlaceId, P};
    use super::trans::{Trans, TransId, T, W1, W2, W3};
    use super::{NetId, PetriNet, N};

    enum P0 {}
    enum P1 {}
    enum T0 {}

    impl<Net: NetId> Place<Net> for P0 {
        const ID: PlaceId<Net> = PlaceId::new(0);
    }
    impl<Net: NetId> Place<Net> for P1 {
        const ID: PlaceId<Net> = PlaceId::new(1);
    }
    impl<Net: NetId> Trans<Net> for T0 {
        const ID: TransId<Net> = TransId::new(2);
    }

    // (p0) -> |t0| -> (p1)
    fn minimal<Id: NetId>() -> PetriNet<Id> {
        PetriNet::empty()
            .add_place::<P0>()
            .add_place::<P1>()
            .add_trans::<T0, (P0, W1), (P1, W1)>()
    }

    // (p0) -\            /-> (p2)
    //        >-> |t0| --<--> (p3)
    // (p1) -/            \-> (p4)
    fn weighted_star<Id: NetId>() -> PetriNet<Id> {
        PetriNet::empty()
            .add_place::<P<0>>()
            .add_place::<P<1>>()
            .add_place::<P<2>>()
            .add_place::<P<3>>()
            .add_place::<P<4>>()
            .add_trans::<T<0>, ((P<0>, W1), (P<1>, W2)), ((P<2>, W1), (P<3>, W2), (P<4>, W3))>()
    }

    // Two places sending a token back and forth through two transitions in opposite directions:
    //  /--> |t0| -> (p1)
    // (p0) <- |t1| <--/
    fn ring<Id: NetId>() -> PetriNet<Id> {
        PetriNet::empty()
            .add_place::<P<0>>()
            .add_place::<P<1>>()
            .add_trans::<T<0>, (P<0>, W1), (P<1>, W1)>()
            .add_trans::<T<1>, (P<1>, W1), (P<0>, W1)>()
    }

    // Two transitions sharing a preset place. When one of them fires, the other ceases to be enabled.
    // (p0) --> |t0| -\
    // (p1) -<         >-> (p3)
    // (p2) --> |t1| -/
    fn choice<Id: NetId>() -> PetriNet<Id> {
        PetriNet::empty()
            .add_place::<P<0>>()
            .add_place::<P<1>>()
            .add_place::<P<2>>()
            .add_place::<P<3>>()
            .add_trans::<T<0>, ((P<0>, W1), (P<1>, W1)), (P<3>, W1)>()
            .add_trans::<T<1>, ((P<1>, W1), (P<2>, W1)), (P<3>, W1)>()
    }

    #[test]
    fn test_minimal() {
        let net = minimal::<N<0>>();
        let mut token = net.spawn_token();
        token.mark::<P0>(1);
        assert!(net.fire::<T0>(&mut token).is_some());
        assert_eq!(token.marks::<P0>(), 0);
        assert_eq!(token.marks::<P1>(), 1);
    }

    #[test]
    fn test_weighted_star() {
        let net = weighted_star::<N<0>>();
        let mut token = net.spawn_token();
        token.mark::<P<0>>(1);
        token.mark::<P<1>>(2);
        assert!(net.fire::<T<0>>(&mut token).is_some());
        assert_eq!(token.marks::<P<0>>(), 0);
        assert_eq!(token.marks::<P<1>>(), 0);
        assert_eq!(token.marks::<P<2>>(), 1);
        assert_eq!(token.marks::<P<3>>(), 2);
        assert_eq!(token.marks::<P<4>>(), 3);
    }

    #[test]
    fn test_ring() {
        let net = ring::<N<0>>();
        let mut token = net.spawn_token();
        token.mark::<P<0>>(1);
        assert_eq!(token.marks::<P<0>>(), 1);
        assert_eq!(token.marks::<P<1>>(), 0);
        assert!(net.fire::<T<0>>(&mut token).is_some());
        assert_eq!(token.marks::<P<0>>(), 0);
        assert_eq!(token.marks::<P<1>>(), 1);
        assert!(net.fire::<T<1>>(&mut token).is_some());
        assert_eq!(token.marks::<P<0>>(), 1);
        assert_eq!(token.marks::<P<1>>(), 0);
    }

    #[test]
    fn test_choice() {
        let net = choice::<N<0>>();
        let mut token = net.spawn_token();
        token.mark::<P<0>>(1);
        token.mark::<P<1>>(1);
        token.mark::<P<2>>(1);
        assert!(net.is_enabled::<T<0>>(&token));
        assert!(net.is_enabled::<T<1>>(&token));
        assert!(net.fire::<T<0>>(&mut token).is_some());
        assert!(!net.is_enabled::<T<1>>(&token));
    }
}
