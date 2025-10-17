use bevy::prelude::*;

#[derive(States, Clone, Eq, PartialEq, Hash, Debug)]
pub enum WorldGenState {
    GenPlates,
    FinishedPlates,
    GenContinents,
    FinishedContinents,
    GenPlateVelocities,
    FinishedPlateVelocities,
    JustChill,
    Finished,
}

#[derive(States, Clone, Eq, PartialEq, Hash, Debug)]
pub enum SimulationState {
    Setup,
    Running,
}

pub struct StatePlugin;

impl Plugin for StatePlugin {
    fn build(&self, app: &mut App) {
        app.insert_state(WorldGenState::GenPlates)
            .insert_state(SimulationState::Setup);
    }
}
