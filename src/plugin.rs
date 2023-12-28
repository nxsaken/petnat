//! Bevy plugin.

use bevy::prelude::{App, Plugin};

/// Petri net plugin.
pub struct PetnatPlugin;

impl Plugin for PetnatPlugin {
    fn build(&self, app: &mut App) {}
}

// TODO: event triggers transition fire

// TODO: event marks place

// TODO: transition sends event

// TODO: place marking sends event
