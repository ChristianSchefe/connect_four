mod board;
mod events;
mod player;
mod visuals;

use board::*;
use events::*;
use player::*;
use visuals::*;

use bevy::prelude::*;
use bevy_tweening::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, TweeningPlugin, PlayerPlugin, EventBusPlugin, VisualsPlugin))
        .insert_resource(Board::new())
        .add_systems(Update, bevy::window::close_on_esc)
        .add_systems(Update, on_game_event)
        .add_systems(Startup, start_game)
        .run();
}

fn start_game(mut commands: Commands, mut writer: EventWriter<GameEvent>, board: Res<Board>) {
    commands.spawn((HumanPlayer { player: Player::PlayerOne },));
    // commands.spawn((AiPlayer { player: Player::PlayerTwo },));
    commands.spawn((AiPlayer { player: Player::PlayerTwo },));
    writer.send(GameEvent::StartGame(board.cur_player))
}

fn on_game_event(mut reader: EventReader<GameEvent>, mut delay_writer: EventWriter<DelayEvent>, mut board: ResMut<Board>) {
    for event in reader.read() {
        info!("Received Game Event: {:?}", event);
        match event {
            GameEvent::DoMove(m) => {
                board.do_move(*m);
                let state = board.get_board_state();
                match state {
                    BoardState::Playing => delay_writer.send(DelayEvent(GameEvent::RequestMove(board.cur_player), 0.1)),
                    BoardState::GameOver(result) => delay_writer.send(DelayEvent(GameEvent::EndGame(result), 0.1)),
                }
            }
            GameEvent::EndGame(_) => delay_writer.send(DelayEvent(GameEvent::ResetBoard, 5.0)),
            GameEvent::StartGame(player) => delay_writer.send(DelayEvent(GameEvent::RequestMove(*player), 0.1)),
            GameEvent::ResetBoard => {
                *board = Board::new();
                delay_writer.send(DelayEvent(GameEvent::StartGame(board.cur_player), 0.1))
            }
            _ => {}
        }
    }
}
