use bevy::{math::U64Vec2, prelude::*};

use crate::GameState;

#[derive(Resource)]
pub struct Board {
    pub size: U64Vec2,
    pub grid: Vec<Option<Player>>,
}

#[derive(Resource)]
pub struct GameStateMachine(pub GameState);


#[derive(Clone, Copy)]
pub enum Player {
    PlayerOne,
    PlaYerTwo,
}
