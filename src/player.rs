use bevy::{
    prelude::*,
    tasks::{block_on, AsyncComputeTaskPool, Task},
    window::PrimaryWindow,
};

use crate::*;
use futures_lite::future;
use rand::{seq::SliceRandom, thread_rng};
use std::{sync::mpsc, thread};

#[derive(Component, Debug)]
pub struct AiPlayer {
    pub player: Player,
}

#[derive(Component, Debug)]
pub struct HumanPlayer {
    pub player: Player,
}

#[derive(Component)]
struct ComputeTask(Task<Move>);

#[derive(Component)]
struct HumanInputListener(Player);

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

#[derive(Resource, Default)]
pub struct WorldCoords(pub Vec2);

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (on_request_move, await_ai_move, calc_world_mouse, await_human_move))
            .init_resource::<WorldCoords>();
    }
}

fn await_ai_move(mut commands: Commands, mut writer: EventWriter<GameEvent>, mut query: Query<(Entity, &mut ComputeTask)>) {
    for (entity, mut task) in &mut query {
        if let Some(computed_move) = block_on(future::poll_once(&mut task.0)) {
            writer.send(GameEvent::DoMove(computed_move));

            commands.entity(entity).remove::<ComputeTask>();
        }
    }
}

fn find_best_move(board: &mut Board) -> Move {
    let (tx, rx) = mpsc::channel();
    let mut rng = thread_rng();
    let mut all_moves: Vec<Move> = board.get_moves();
    all_moves.shuffle(&mut rng);

    let mut handles = Vec::new();

    for &m in all_moves.iter() {
        board.do_move(m);
        let mut board_clone = board.clone();
        board.undo_move();

        let tx_clone = tx.clone();
        let handle = thread::spawn(move || {
            let evaluation = -evaluate_move(&mut board_clone, 7);
            tx_clone.send((m, evaluation)).unwrap();
        });
        handles.push(handle);
    }

    let mut best_move = all_moves[0];
    let mut best_evaluation = f32::MIN;

    for _ in 0..all_moves.len() {
        if let Ok((m, eval)) = rx.recv() {
            debug!("{:?} is {}", m, eval);
            if eval > best_evaluation {
                best_evaluation = eval;
                best_move = m
            }
        }
    }

    if best_evaluation < -50.0 {
        warn!("forced loss for {:?}!", best_move.player);
    } else if best_evaluation > 50.0 {
        warn!("forced win for {:?}!", best_move.player);
    }

    best_move
}

fn evaluate_move(board: &mut Board, depth: u32) -> f32 {
    if board.check_for_win().is_some() {
        -100.0 - depth as f32
    } else if depth == 0 {
        0.0
    } else {
        board
            .get_moves()
            .iter()
            .map(|&m| {
                board.do_move(m);
                let evaluation = -evaluate_move(board, depth - 1);
                board.undo_move();
                evaluation
            })
            .reduce(f32::max)
            .unwrap_or(0.0)
    }
}

fn on_request_move(mut commands: Commands, mut reader: EventReader<GameEvent>, human_query: Query<&HumanPlayer>, ai_query: Query<&AiPlayer>, board: Res<Board>) {
    for event in reader.read() {
        if let GameEvent::RequestMove(player) = event {
            if let Some(human) = human_query.iter().find(|&human| human.player == *player) {
                commands.spawn(HumanInputListener(human.player));
            }
            if let Some(_ai) = ai_query.iter().find(|&ai| ai.player == *player) {
                let pool = AsyncComputeTaskPool::get();

                let mut board_clone = board.clone();
                let task = pool.spawn(async move { find_best_move(&mut board_clone) });
                commands.spawn(ComputeTask(task));
            }
        }
    }
}

fn await_human_move(
    mut commands: Commands,
    input: Res<Input<MouseButton>>,
    board: Res<Board>,
    mouse_position: Res<WorldCoords>,
    mut writer: EventWriter<GameEvent>,
    query: Query<(Entity, &HumanInputListener)>,
) {
    if let Ok((entity, player)) = query.get_single() {
        if input.just_released(MouseButton::Left) {
            if let Some(grid_pos) = board.world_to_grid(mouse_position.0) {
                let m = Move { player: player.0, pos: grid_pos };
                if board.is_valid_move(m) {
                    writer.send(GameEvent::DoMove(m));
                    commands.entity(entity).despawn();
                }
            }
        }
    }
}

fn calc_world_mouse(mut world_coords: ResMut<WorldCoords>, window_query: Query<&Window, With<PrimaryWindow>>, cam_query: Query<(&Camera, &GlobalTransform), With<MainCamera>>) {
    let (camera, camera_transform) = cam_query.single();
    let window = window_query.single();

    if let Some(world_position) = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
        .map(|ray| ray.origin.truncate())
    {
        world_coords.0 = world_position;
    }
}
