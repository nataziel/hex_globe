use bevy::prelude::*;

#[derive(States, Clone, Eq, PartialEq, Hash, Debug)]
pub enum GameState {
    WorldGen,
    Simulation,
}

#[derive(SubStates, Clone, Eq, PartialEq, Hash, Debug, Default)]
#[source(GameState = GameState::WorldGen)]
pub enum WorldGenState {
    #[default]
    SeedPlates,
    GenPlates,
    FinishedPlates,
    AssignPlateBoundaries,
    FinishedPlateBoundaries,
    GenContinents,
    FinishedContinents,
    GenPlateVelocities,
    FinishedPlateVelocities,
    JustChill,
    Finished,
}

#[derive(SubStates, Clone, Eq, PartialEq, Hash, Debug, Default)]
#[source(GameState = GameState::Simulation)]
pub enum SimulationState {
    #[default]
    Running,
}

pub struct StatePlugin;

impl Plugin for StatePlugin {
    fn build(&self, app: &mut App) {
        app.insert_state(GameState::WorldGen)
            .add_sub_state::<WorldGenState>()
            .add_sub_state::<SimulationState>();
    }
}
