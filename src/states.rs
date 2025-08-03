use bevy::prelude::*;

#[derive(States, Clone, Eq, PartialEq, Hash, Debug)]
pub enum WorldGenState {
    GenPlates,
    GenContinents,
    JustChill,
}

