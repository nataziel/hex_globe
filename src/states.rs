use bevy::prelude::*;

#[derive(States, Clone, Eq, PartialEq, Hash, Debug)]
pub enum WorldGenState {
    GenPlates,
    GenContinents,
    JustChill,
}

pub struct StatePlugin;

impl Plugin for StatePlugin {
    fn build(&self, app: &mut App) {
        app.insert_state(WorldGenState::GenPlates);
    }
}
