//! Bevy plugin.

use crate::net::place::Place;
use crate::net::trans::Trans;
use crate::net::{NetId, PetriNet};
use crate::token::Token;
use std::marker::PhantomData;

/// TODO: flesh out plugin api
pub struct PetriNetPlugin<Net: NetId> {
    _net: PhantomData<Net>,
}

/// TODO: how to resolve choices?
fn _transition_system<Net: NetId>(net: PetriNet<Net>, mut tokens: Vec<&mut Token<Net>>) {
    for token in &mut tokens {
        let enabled = net.iter_enabled(token).collect::<Vec<_>>();
        for trans in enabled {
            net.fire_by_id(trans, token);
        }
    }
}

/// TODO: Possible event for queueing an attempt to fire a Petri net.
pub struct TransQueueEvent<Net: NetId, T: Trans<Net>>(PhantomData<(Net, T)>);

fn _example_immediate_event<Net: NetId, T: Trans<Net>>(
    queue_events: Vec<TransQueueEvent<Net, T>>,
    net: PetriNet<Net>,
    mut tokens: Vec<&mut Token<Net>>,
) {
    for _ in queue_events.iter() {
        for token in &mut tokens {
            net.fire::<T>(token);
        }
    }
}

/// TODO: Possible event for queueing a marking of a Petri net place.
pub struct PlaceMarkEvent<Net: NetId, P: Place<Net>>(usize, PhantomData<(Net, P)>);

fn _example_mark_place_with_event<Net: NetId, P: Place<Net>>(
    mark_events: Vec<PlaceMarkEvent<Net, P>>,
    mut tokens: Vec<&mut Token<Net>>,
) {
    for PlaceMarkEvent(n, _) in mark_events.iter() {
        for token in &mut tokens {
            token.mark::<P>(*n);
        }
    }
}

/// TODO: Possible event signifying that a Petri net transition has been fired.
///   unclear how to do this properly, do we generate an event per transition always, no matter who fired it?
pub struct TransEvent<Net: NetId, T: Trans<Net>>(PhantomData<(Net, T)>);

// TODO: place marking sends event - same as the previous issue
