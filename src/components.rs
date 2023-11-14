use bevy::prelude::*;

use crate::GameState;

#[derive(Resource)]
pub struct Board {
    pub size: UVec2,
    pub grid: Vec<Option<Player>>,
}

#[derive(Resource)]
pub struct GameStateMachine(pub GameState);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Player {
    PlayerOne,
    PlayerTwo,
}

impl Board {
    pub fn grid_to_world(self: &Board, grid_pos: UVec2) -> Vec2 {
        self.vec2_to_world(grid_pos.as_vec2())
    }

    pub fn vec2_to_world(self: &Board, grid_pos: Vec2) -> Vec2 {
        grid_pos - (self.size - UVec2::ONE).as_vec2() * 0.5
    }

    pub fn world_to_grid(self: &Board, world_pos: Vec2) -> Option<UVec2> {
        let pos = (world_pos + (self.size - UVec2::ONE).as_vec2() * 0.5)
            .round()
            .as_ivec2();
        if self.valid_ivec_pos(pos) {
            Some(pos.as_uvec2())
        } else {
            None
        }
    }

    pub fn valid_ivec_pos(self: &Board, pos: IVec2) -> bool {
        pos.x >= 0 && pos.y >= 0 && (pos.x as u32) < self.size.x && (pos.y as u32) < self.size.y
    }

    pub fn valid_uvec_pos(self: &Board, pos: UVec2) -> bool {
        pos.x < self.size.x && pos.y < self.size.y
    }

    pub fn set(self: &mut Board, grid_pos: UVec2, value: Option<Player>) {
        let index = (grid_pos.x + grid_pos.y * self.size.x) as usize;
        self.grid[index] = value;

        debug!("Set {}({}) to {:?}", grid_pos, index, value);
    }

    pub fn get(self: &Board, grid_pos: UVec2) -> Option<Player> {
        let index = (grid_pos.x + grid_pos.y * self.size.x) as usize;

        if self.valid_uvec_pos(grid_pos) {
            debug!("Get {}({}) -> {:?}", grid_pos, index, self.grid[index]);
            self.grid[index]
        } else {
            debug!("Get {}({}) -> Invalid Position", grid_pos, index);
            None
        }
    }
}
