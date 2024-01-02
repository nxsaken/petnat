//! Bevy plugin.

use std::marker::PhantomData;

use bevy_app::{App, Plugin};
use bevy_ecs::change_detection::DetectChangesMut;
use bevy_ecs::system::{Query, Res};

use crate::net::place::Place;
use crate::net::trans::Trans;
use crate::net::{NetId, PetriNet};
use crate::token::Token;
use crate::PetriNetBuilder;

/// Plugin that initializes and manages a [`PetriNet`].
pub struct PetriNetPlugin<Net: NetId> {
    /// Function used to build the [`PetriNet`].
    /// FIXME: feels clunky?
    pub build: fn(PetriNetBuilder<Net>) -> PetriNetBuilder<Net>,
}

impl<Net: NetId> Plugin for PetriNetPlugin<Net> {
    fn build(&self, app: &mut App) {
        let builder = PetriNet::builder();
        let pnet = (self.build)(builder).build();
        app.insert_resource(pnet);
    }
}

/// TODO: how to resolve choices?
fn _transition_system<Net: NetId>(net: Res<PetriNet<Net>>, mut tokens: Query<&mut Token<Net>>) {
    for mut token in &mut tokens {
        let fired = net.fire_all(token.bypass_change_detection());
        if !fired.is_empty() {
            token.set_changed()
        }
    }
}

/// TODO: Possible event for queueing an attempt to fire a Petri net.
pub struct _TransQueueEvent<Net: NetId, T: Trans<Net>>(PhantomData<(Net, T)>);

fn _example_immediate_event<Net: NetId, T: Trans<Net>>(
    queue_events: Vec<_TransQueueEvent<Net, T>>,
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
pub struct _PlaceMarkEvent<Net: NetId, P: Place<Net>>(usize, PhantomData<(Net, P)>);

fn _example_mark_place_with_event<Net: NetId, P: Place<Net>>(
    mark_events: Vec<_PlaceMarkEvent<Net, P>>,
    net: PetriNet<Net>,
    mut tokens: Vec<&mut Token<Net>>,
) {
    for _PlaceMarkEvent(n, _) in mark_events.iter() {
        for token in &mut tokens {
            net.mark::<P>(token, *n);
        }
    }
}

/// TODO: Possible event signifying that a Petri net transition has been fired.
///   unclear how to do this properly, do we generate an event per transition always, no matter who fired it?
pub struct _TransEvent<Net: NetId, T: Trans<Net>>(PhantomData<(Net, T)>);

// TODO: place marking sends event - same as the previous issue
