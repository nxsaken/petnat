//! A simple Petri net being manipulated via systems.

use bevy::input::common_conditions::input_just_pressed;
use bevy::prelude::*;
use petnat::{NetId, Nn, PetriNet, PetriNetPlugin, Place, Pn, Tn, Token, W};
use std::any::type_name;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // (P0) -\ 1       1
        //        >-> |T0| -> (P2)
        // (P1) -/ 2
        .add_plugins(PetriNetPlugin::<Nn<0>> {
            build: |builder| {
                builder
                    .add_place::<Pn<0>>()
                    .add_place::<Pn<1>>()
                    .add_place::<Pn<2>>()
                    // T0 requires 1 token in P0 and 2 tokens in P1 to be enabled
                    // and it will produce 1 token in P2 when fired
                    .add_trans::<Tn<0>, ((Pn<0>, W<1>), (Pn<1>, W<2>)), (Pn<2>, W<1>)>()
            },
        })
        .add_systems(Startup, spawn_token::<Nn<0>>)
        .add_systems(
            Update,
            (
                // press 1 and 2 to mark `P0` and `P1`
                mark::<Nn<0>, Pn<0>>.run_if(input_just_pressed(KeyCode::Key1)),
                mark::<Nn<0>, Pn<1>>.run_if(input_just_pressed(KeyCode::Key2)),
                // press T to fire `T0`
                trans_t0::<Nn<0>>.run_if(input_just_pressed(KeyCode::T)),
                print_net::<Nn<0>>,
            ),
        )
        .run();
}

fn spawn_token<Net: NetId>(mut commands: Commands, net: Res<PetriNet<Net>>) {
    // A token can represent the current state of some game object,
    // or some sort of resource
    commands.spawn(net.spawn_token());
    info!("Spawning a token...");
}

fn mark<Net: NetId, P: Place<Net>>(net: Res<PetriNet<Net>>, mut tokens: Query<&mut Token<Net>>) {
    for mut token in &mut tokens {
        net.mark::<P>(&mut token, 1);
        // TODO: better place/trans names
        let (_, name) = type_name::<P>()
            .rsplit_once(':')
            .unwrap_or(("", type_name::<P>()));
        info!("{} marked!", name);
    }
}

fn trans_t0<Net: NetId>(net: Res<PetriNet<Net>>, mut tokens: Query<&mut Token<Net>>) {
    for mut token in &mut tokens {
        // TODO: better handling of change detection
        if let Some(()) = net.fire::<Tn<0>>(token.bypass_change_detection()) {
            info!("T0 fired!");
            token.set_changed();
        } else {
            info!("T0 cannot fire! (Need: 1 in P0 + 2 in P1)");
        }
    }
}

fn print_net<Net: NetId>(
    net: Res<PetriNet<Net>>,
    tokens: Query<(Entity, &Token<Net>), Changed<Token<Net>>>,
) {
    for (id, token) in &tokens {
        info!("== TOKEN {:?} STATE ==", id);
        info!("P0: {}", token.marks::<Pn<0>>());
        info!("P1: {}", token.marks::<Pn<1>>());
        info!("T0 enabled: {}", net.enabled::<Tn<0>>(token));
        info!("P2: {}", token.marks::<Pn<2>>());
        info!("=====================");
    }
}
