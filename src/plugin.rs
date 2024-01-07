//! Bevy plugin.

use bevy_app::{App, Plugin};

use crate::net::{NetId, PetriNet};

/// Plugin that initializes and manages a [`PetriNet`].
pub struct PetriNetPlugin<Net: NetId> {
    /// Function used to build the [`PetriNet`].
    /// FIXME: feels clunky?
    pub build: fn(PetriNet<Net>) -> PetriNet<Net>,
}

impl<Net: NetId> Plugin for PetriNetPlugin<Net> {
    fn build(&self, app: &mut App) {
        let pnet = (self.build)(PetriNet::new());
        app.insert_resource(pnet);
    }
}
