mod components;

use bevy::{prelude::*, render::camera::ScalingMode, window::PrimaryWindow};
use bevy_tweening::{
    component_animator_system, lens::SpriteColorLens, AnimationSystem, Animator, EaseFunction,
    Tween, TweeningPlugin,
};
use components::{Board, GameStateMachine};

use crate::components::Player;

const BACKGROUND_COLOR: Color = Color::rgb(0.9, 0.9, 0.9);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameState {
    PlayerOneTurn,
    PlayerTwoTurn,
    GameOver,
}

#[derive(Event)]
struct EndTurnEvent(pub GameState);

#[derive(Resource, Default)]
struct WorldCoords(Vec2);

#[derive(Component, Debug)]
struct GridPosition(UVec2);

#[derive(Component)]
struct MainCamera;

#[derive(Component)]
struct AnimationTarget(Option<Player>);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(TweeningPlugin)
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .insert_resource(create_board_resource())
        .add_systems(
            Update,
            component_animator_system::<Sprite>.in_set(AnimationSystem::AnimationUpdate),
        )
        .add_systems(Startup, (setup_camera, setup_board))
        .add_systems(Update, bevy::window::close_on_esc)
        .add_systems(Update, (do_move, end_turn, calc_world_mouse, update_tile))
        .insert_resource(GameStateMachine(GameState::PlayerOneTurn))
        .init_resource::<WorldCoords>()
        .add_event::<EndTurnEvent>()
        .run();
}

fn create_board_resource() -> Board {
    let size = UVec2 { x: 7, y: 6 };
    let board = Board {
        size,
        grid: vec![None; (size.x * size.y) as usize],
    };
    board
}

fn setup_camera(mut commands: Commands) {
    let mut cam = Camera2dBundle::default();
    cam.projection.scaling_mode = ScalingMode::FixedVertical(10f32);

    commands.spawn((cam, MainCamera));
}

fn setup_board(mut commands: Commands, board: Res<Board>) {
    for y in 0..board.size.y {
        for x in 0..board.size.x {
            let pos = UVec2 { x, y };
            commands.spawn((
                SpriteBundle {
                    transform: Transform {
                        translation: board.grid_to_world(pos).extend(0.0),
                        scale: Vec3::new(0.9, 0.9, 1.0),
                        ..default()
                    },
                    sprite: Sprite {
                        color: Color::rgb(1.0, 1.0, 1.0),
                        ..default()
                    },
                    ..default()
                },
                GridPosition(pos),
                AnimationTarget(None),
            ));
        }
    }
}

fn calc_world_mouse(
    mut world_coords: ResMut<WorldCoords>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    cam_query: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
) {
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

fn do_move(
    input: Res<Input<MouseButton>>,
    game_state: Res<GameStateMachine>,
    mut board: ResMut<Board>,
    mouse_position: Res<WorldCoords>,
    mut ev_end_turn: EventWriter<EndTurnEvent>,
) {
    if input.just_released(MouseButton::Left) {
        info!(
            "World coords: {}/{}",
            mouse_position.0.x, mouse_position.0.y
        );
        if let Some(grid_pos) = board.world_to_grid(mouse_position.0) {
            info!("Grid pos: {}/{}", grid_pos.x, grid_pos.y);
            debug!("End Turn Event sent");

            let cur_player = match &game_state.0 {
                GameState::PlayerOneTurn => Some(Player::PlayerOne),
                GameState::PlayerTwoTurn => Some(Player::PlayerTwo),
                GameState::GameOver => None,
            };

            if board.get(grid_pos).is_none()
                && (grid_pos.y == 0 || board.get(grid_pos - UVec2::new(0, 1)).is_some())
            {
                board.set(grid_pos, cur_player);
                ev_end_turn.send(EndTurnEvent(game_state.0));
            }
            info!("Can't place here");
        }
    }
}

fn update_tile(
    mut commands: Commands,
    mut tiles: Query<(&GridPosition, &Sprite, &mut AnimationTarget, Entity)>,
    board: Res<Board>,
) {
    for (pos, sprite, mut animation_target, entity) in tiles.iter_mut() {
        let tile_type = board.get(pos.0);
        let end_color = match tile_type {
            Some(Player::PlayerOne) => Color::BLUE,
            Some(Player::PlayerTwo) => Color::RED,
            None => Color::WHITE,
        };
        if tile_type == animation_target.0 {
            continue;
        }
        animation_target.0 = tile_type;

        let tween = Tween::new(
            EaseFunction::CubicInOut,
            std::time::Duration::from_secs(1),
            SpriteColorLens {
                start: sprite.color,
                end: end_color,
            },
        );
        info!("Add animator at {:?}", pos);
        commands.entity(entity).remove::<Animator<Sprite>>();
        commands.entity(entity).insert(Animator::new(tween));
    }
}

fn end_turn(mut ev_end_turn: EventReader<EndTurnEvent>, mut game_state: ResMut<GameStateMachine>) {
    for ev in ev_end_turn.read() {
        if ev.0 != game_state.0 {
            warn!("End turn from wrong Player!");
            continue;
        }
        game_state.0 = match &game_state.0 {
            GameState::PlayerOneTurn => GameState::PlayerTwoTurn,
            GameState::PlayerTwoTurn => GameState::PlayerOneTurn,
            GameState::GameOver => GameState::GameOver,
        };

        info!("End Turn from {:?}. New State: {:?}", ev.0, game_state.0);
    }
}
