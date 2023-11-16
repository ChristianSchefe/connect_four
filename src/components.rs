use bevy::prelude::*;
use bevy_tweening::Lens;

use crate::DIRECTIONS;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameState {
    PlayerOneTurn,
    PlayerTwoTurn,
    GameOver,
}

#[derive(Event)]
pub struct EndTurnEvent(pub GameState, pub UVec2);

#[derive(Event, Debug)]
pub struct WinEvent {
    pub winning_player: Player,
    pub pos: UVec2,
    pub dir: IVec2,
}

#[derive(Resource, Default)]
pub struct WorldCoords(pub Vec2);

#[derive(Component, Debug)]
pub struct GridPosition(pub UVec2);

#[derive(Component)]
pub struct MainCamera;

#[derive(Component)]
pub struct AnimationTarget(pub Option<Player>);

#[derive(Component)]
pub struct TileMarker;

pub struct BackgroundColorLens {
    pub start: Color,
    pub end: Color,
}

impl Lens<BackgroundColor> for BackgroundColorLens {
    fn lerp(&mut self, target: &mut BackgroundColor, ratio: f32) {
        let start = Vec4::from(self.start);
        let end = Vec4::from(self.end);
        *target = Color::from(Vec4::lerp(start, end, ratio)).into();
    }
}

impl Board {
    pub fn get_offset(&self) -> Vec2 {
        (self.size - UVec2::ONE).as_vec2() * 0.5 + Vec2::new(0.0, 0.0)
    }

    pub fn grid_to_world(&self, grid_pos: UVec2) -> Vec2 {
        self.vec2_to_world(grid_pos.as_vec2())
    }

    pub fn vec2_to_world(&self, grid_pos: Vec2) -> Vec2 {
        grid_pos - self.get_offset()
    }

    pub fn world_to_grid(&self, world_pos: Vec2) -> Option<UVec2> {
        let pos = (world_pos + self.get_offset()).round().as_ivec2();
        if self.valid_ivec_pos(pos) {
            Some(pos.as_uvec2())
        } else {
            None
        }
    }

    pub fn valid_ivec_pos(&self, pos: IVec2) -> bool {
        pos.x >= 0 && pos.y >= 0 && (pos.x as u32) < self.size.x && (pos.y as u32) < self.size.y
    }

    pub fn valid_uvec_pos(self: &Board, pos: UVec2) -> bool {
        pos.x < self.size.x && pos.y < self.size.y
    }

    pub fn set(&mut self, grid_pos: UVec2, value: Option<Player>) {
        let index = (grid_pos.x + grid_pos.y * self.size.x) as usize;
        self.grid[index] = value;

        debug!("Set {}({}) to {:?}", grid_pos, index, value);
    }

    pub fn get(&self, grid_pos: UVec2) -> Option<Player> {
        let index = (grid_pos.x + grid_pos.y * self.size.x) as usize;

        if self.valid_uvec_pos(grid_pos) {
            debug!("Get {}({}) -> {:?}", grid_pos, index, self.grid[index]);
            self.grid[index]
        } else {
            debug!("Get {}({}) -> Invalid Position", grid_pos, index);
            None
        }
    }

    pub fn check_for_win(&self, team: Player, updated_pos: UVec2) -> Option<IVec2> {
        let check_dir = |dir: IVec2| {
            let mut has_four = true;
            for i in 0..4 {
                let pos = updated_pos.as_ivec2() + dir * i;

                if !self.valid_ivec_pos(pos) || !self.get(pos.as_uvec2()).is_some_and(|p| p == team) {
                    has_four = false;
                    break;
                }
            }
            if has_four {
                return true;
            }
            false
        };

        DIRECTIONS.iter().find(|&&dir| check_dir(dir)).copied()
    }
}
