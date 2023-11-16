use bevy::prelude::*;
use bevy_tweening::Lens;

use crate::WIN_DIRECTIONS;

#[derive(Resource, Clone)]
pub struct Board {
    pub size: UVec2,
    pub grid: Vec<Option<Player>>,
    pub levels: Vec<u32>,
}

#[derive(Resource)]
pub struct GameStateMachine(pub GameState);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Player {
    PlayerOne,
    PlayerTwo,
}

impl Player {
    pub fn opposite(self) -> Self {
        match self {
            Player::PlayerOne => Player::PlayerTwo,
            Player::PlayerTwo => Player::PlayerOne,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameState {
    PlayerOneTurn,
    PlayerTwoTurn,
    GameOver,
}

#[derive(Event)]
pub struct EndTurnEvent(pub GameState, pub UVec2);

#[derive(Event)]
pub struct RequestMoveEvent(pub Player);

#[derive(Event, Debug)]
pub struct WinEvent {
    pub winning_player: Player,
    pub from_pos: UVec2,
    pub to_pos: UVec2,
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

#[derive(Debug, Clone, Copy)]
pub struct Move {
    pub pos: UVec2,
    pub player: Player,
}

impl Lens<BackgroundColor> for BackgroundColorLens {
    fn lerp(&mut self, target: &mut BackgroundColor, ratio: f32) {
        let start = Vec4::from(self.start);
        let end = Vec4::from(self.end);
        *target = Color::from(Vec4::lerp(start, end, ratio)).into();
    }
}

impl Board {
    pub fn new() -> Self {
        let size = UVec2::new(7, 6);
        Board {
            size,
            grid: vec![None; (size.x * size.y) as usize],
            levels: vec![0; size.x as usize],
        }
    }
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

    fn set(&mut self, grid_pos: UVec2, value: Option<Player>) {
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

    pub fn check_for_win(&self, team: Player, updated_pos: UVec2) -> Option<(UVec2, UVec2)> {
        let check_dir = |dir: IVec2| {
            let mut fwd_count = 0;
            let mut bwd_count = 0;
            for i in 1..4 {
                let pos = updated_pos.as_ivec2() + dir * i;

                if !self.valid_ivec_pos(pos) || !self.get(pos.as_uvec2()).is_some_and(|p| p == team) {
                    break;
                }
                fwd_count += 1
            }
            for i in 1..4 {
                let pos = updated_pos.as_ivec2() - dir * i;

                if !self.valid_ivec_pos(pos) || !self.get(pos.as_uvec2()).is_some_and(|p| p == team) {
                    break;
                }
                bwd_count += 1
            }
            if fwd_count + bwd_count >= 3 {
                if fwd_count >= bwd_count {
                    Some(((updated_pos.as_ivec2() + dir * fwd_count).as_uvec2(), (updated_pos.as_ivec2() - dir * bwd_count).as_uvec2()))
                } else {
                    Some(((updated_pos.as_ivec2() - dir * bwd_count).as_uvec2(), (updated_pos.as_ivec2() + dir * fwd_count).as_uvec2()))
                }
            } else {
                None
            }
        };

        WIN_DIRECTIONS.iter().find_map(|&dir| check_dir(dir))
    }

    pub fn do_move(&mut self, board_move: Move) {
        if self.valid_uvec_pos(board_move.pos) && self.get(board_move.pos).is_none() && board_move.pos.y == self.levels[board_move.pos.x as usize] {
            self.set(board_move.pos, Some(board_move.player));
            self.levels[board_move.pos.x as usize] += 1;
        } else {
            error!("Player {:?} Tried Invalid Move!", board_move.player);
        }
    }

    pub fn undo_move(&mut self, board_move: Move) {
        self.set(board_move.pos, None);
        self.levels[board_move.pos.x as usize] -= 1;
    }

    pub fn is_draw(&self) -> bool {
        !self.levels.iter().min().is_some_and(|n| n + 1 < self.size.y)
    }
}
