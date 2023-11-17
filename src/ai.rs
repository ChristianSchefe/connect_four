use bevy::{
    prelude::*,
    tasks::{block_on, AsyncComputeTaskPool, Task},
};
// use bevy_tasks::{AsyncComputeTaskPool, Task};
use crate::components::*;
use futures_lite::future;
use rand::{seq::SliceRandom, thread_rng};
use std::{sync::mpsc, thread};

#[derive(Component, Debug)]
pub struct Ai {
    pub player: Player,
}

#[derive(Component)]
struct ComputeTask(Task<Move>);

pub struct AiPlugin;

impl Plugin for AiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (listen_for_turn, handle_tasks));
    }
}

pub fn listen_for_turn(mut commands: Commands, mut request_move_event: EventReader<RequestMoveEvent>, ai_query: Query<&Ai>, board: Res<Board>) {
    for ev in request_move_event.read() {
        if let Some(ai) = ai_query.iter().find(|&ai| ai.player == ev.0) {
            // info!("Start move gen task for ai");

            let pool = AsyncComputeTaskPool::get();

            let mut board_clone = board.clone();
            let player = ai.player.clone();
            let task = pool.spawn(async move { find_best_move(&mut board_clone, player) });
            commands.spawn(ComputeTask(task));
        }
    }
}

fn handle_tasks(
    mut commands: Commands,
    mut board: ResMut<Board>,
    game_state: Res<GameStateMachine>,
    mut end_turn_event: EventWriter<EndTurnEvent>,
    mut transform_tasks: Query<(Entity, &mut ComputeTask)>,
) {
    for (entity, mut task) in &mut transform_tasks {
        if let Some(computed_move) = block_on(future::poll_once(&mut task.0)) {
            // info!("Task finished");
            board.do_move(computed_move);
            end_turn_event.send(EndTurnEvent(game_state.0, computed_move.pos));

            commands.entity(entity).remove::<ComputeTask>();
        }
    }
}

pub fn find_best_move(board: &mut Board, player: Player) -> Move {
    let (tx, rx) = mpsc::channel();
    let mut rng = thread_rng();
    let mut all_moves = get_moves(&board, player);
    all_moves.shuffle(&mut rng);

    let mut handles = Vec::new();

    for &m in all_moves.iter() {
        board.do_move(m);
        let mut board_clone = board.clone();
        board.undo_move(m);

        let tx_clone = tx.clone();
        let handle = thread::spawn(move || {
            let evaluation = -evaluate_move(&mut board_clone, 7, m);
            tx_clone.send((m, evaluation)).unwrap();
        });
        handles.push(handle);
    }

    let mut best_move = all_moves[0];
    let mut best_evaluation = f32::MIN;

    for _ in 0..all_moves.len() {
        if let Ok((m, eval)) = rx.recv() {
            info!("{:?} is {}", m, eval);
            if eval > best_evaluation {
                best_evaluation = eval;
                best_move = m
            }
        }
    }

    best_move
}

fn evaluate_move(board: &mut Board, depth: u32, last_move: Move) -> f32 {
    if board.check_for_win(last_move.player, last_move.pos).is_some() {
        -100.0 - depth as f32
    } else if depth == 0 {
        evaluate_board(board, last_move)
    } else {
        get_moves(board, last_move.player.opposite())
            .iter()
            .map(|&m| {
                board.do_move(m);
                let evaluation = -evaluate_move(board, depth - 1, m);
                board.undo_move(m);
                evaluation
            })
            .reduce(f32::max)
            .unwrap_or(0.0)
    }
}

fn evaluate_board(_board: &mut Board, _last_move: Move) -> f32 {
    0.0
}

fn get_moves(board: &Board, player: Player) -> Vec<Move> {
    board
        .levels
        .iter()
        .enumerate()
        .filter(|(_, &y)| y < board.size.y)
        .map(|(i, &y)| Move {
            pos: UVec2::new(i as u32, y),
            player,
        })
        .collect()
}
