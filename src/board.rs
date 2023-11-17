use crate::player::Player;

use bevy::prelude::*;

const WIN_DIRECTIONS: [IVec2; 4] = [IVec2::new(1, 0), IVec2::new(1, 1), IVec2::new(0, 1), IVec2::new(-1, 1)];

#[derive(Resource, Clone)]
pub struct Board {
    pub size: UVec2,
    pub grid: Vec<Option<Player>>,
    pub levels: Vec<u32>,
    pub move_history: Vec<Move>,
    pub cur_player: Player,
}

impl Board {
    pub fn new() -> Self {
        let size = UVec2::new(7, 6);
        Board {
            size,
            grid: vec![None; (size.x * size.y) as usize],
            levels: vec![0; size.x as usize],
            move_history: Vec::with_capacity((size.x * size.y) as usize),
            cur_player: Player::PlayerOne,
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

    pub fn check_for_win(&self) -> Option<WinningLine> {
        if let Some(m) = self.move_history.last() {
            let check_dir = |dir: IVec2| {
                let mut fwd_count = 0;
                let mut bwd_count = 0;
                for i in 1..4 {
                    let pos = m.pos.as_ivec2() + dir * i;

                    if !self.valid_ivec_pos(pos) || !self.get(pos.as_uvec2()).is_some_and(|p| p == m.player) {
                        break;
                    }
                    fwd_count += 1
                }
                for i in 1..4 {
                    let pos = m.pos.as_ivec2() - dir * i;

                    if !self.valid_ivec_pos(pos) || !self.get(pos.as_uvec2()).is_some_and(|p| p == m.player) {
                        break;
                    }
                    bwd_count += 1
                }
                if fwd_count + bwd_count >= 3 {
                    if fwd_count >= bwd_count {
                        Some(WinningLine(
                            (m.pos.as_ivec2() + dir * fwd_count).as_uvec2(),
                            (m.pos.as_ivec2() - dir * bwd_count).as_uvec2(),
                        ))
                    } else {
                        Some(WinningLine(
                            (m.pos.as_ivec2() - dir * bwd_count).as_uvec2(),
                            (m.pos.as_ivec2() + dir * fwd_count).as_uvec2(),
                        ))
                    }
                } else {
                    None
                }
            };

            WIN_DIRECTIONS.iter().find_map(|&dir| check_dir(dir))
        } else {
            None
        }
    }

    pub fn is_valid_move(&self, board_move: Move) -> bool {
        board_move.player == self.cur_player
            && self.valid_uvec_pos(board_move.pos)
            && self.get(board_move.pos).is_none()
            && board_move.pos.y == self.levels[board_move.pos.x as usize]
    }

    pub fn do_move(&mut self, board_move: Move) {
        self.set(board_move.pos, Some(board_move.player));
        self.levels[board_move.pos.x as usize] += 1;
        self.move_history.push(board_move);
        self.cur_player = self.cur_player.opposite();
    }

    pub fn undo_move(&mut self) {
        if let Some(board_move) = self.move_history.pop() {
            self.set(board_move.pos, None);
            self.levels[board_move.pos.x as usize] -= 1;
            self.cur_player = self.cur_player.opposite();
        }
    }

    pub fn is_draw(&self) -> bool {
        self.levels.iter().min().is_some_and(|&n| n >= self.size.y)
    }

    pub fn get_moves(&self) -> Vec<Move> {
        self.levels
            .iter()
            .enumerate()
            .filter(|(_, &y)| y < self.size.y)
            .map(|(i, &y)| Move {
                pos: UVec2::new(i as u32, y),
                player: self.cur_player,
            })
            .collect()
    }

    pub fn get_board_state(&self) -> BoardState {
        if self.is_draw() {
            BoardState::GameOver(GameResult::Draw)
        } else if let Some(winning_line) = self.check_for_win() {
            BoardState::GameOver(GameResult::Win(self.cur_player.opposite(), winning_line))
        } else {
            BoardState::Playing
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum BoardState {
    Playing,
    GameOver(GameResult),
}

#[derive(Debug, Clone, Copy)]
pub struct Move {
    pub pos: UVec2,
    pub player: Player,
}

#[derive(Debug, Clone, Copy)]
pub enum GameResult {
    Win(Player, WinningLine),
    Draw,
}

#[derive(Debug, Clone, Copy)]
pub struct WinningLine(pub UVec2, pub UVec2);
