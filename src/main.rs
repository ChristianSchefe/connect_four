mod components;

use bevy::{math::U64Vec2, prelude::*, render::camera::ScalingMode, window::PrimaryWindow};
use components::{Board, GameStateMachine};

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

#[derive(Component)]
struct MainCamera;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .add_systems(Startup, setup)
        .add_systems(Update, bevy::window::close_on_esc)
        .add_systems(Update, (do_move, end_turn, calc_world_mouse))
        .insert_resource(GameStateMachine(GameState::PlayerOneTurn))
        .add_event::<EndTurnEvent>()
        .run();
}

fn setup(mut commands: Commands) {
    let size = U64Vec2 { x: 7, y: 6 };
    commands.insert_resource(Board {
        size,
        grid: vec![None; (size.x * size.y) as usize],
    });
    commands.init_resource::<WorldCoords>();

    let mut cam = Camera2dBundle::default();
    cam.projection.scaling_mode = ScalingMode::FixedVertical(10f32);

    commands.spawn((cam, MainCamera));

    commands.spawn((SpriteBundle {
        transform: Transform {
            translation: Vec3::new(0.0, 0.0, 0.0),
            scale: Vec3::new(1.0, 1.0, 1.0),
            ..default()
        },
        sprite: Sprite {
            color: Color::rgb(1.0, 1.0, 1.0),
            ..default()
        },
        ..default()
    },));
}

fn calc_world_mouse(
    mut mycoords: ResMut<WorldCoords>,
    q_window: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
) {
    let (camera, camera_transform) = q_camera.single();
    let window = q_window.single();

    if let Some(world_position) = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
        .map(|ray| ray.origin.truncate())
    {
        mycoords.0 = world_position;
        info!("World coords: {}/{}", world_position.x, world_position.y);
    }
}

fn do_move(
    input: Res<Input<MouseButton>>,
    game_state: Res<GameStateMachine>,
    board: Res<Board>,
    mut ev_end_turn: EventWriter<EndTurnEvent>,
) {
    if input.just_released(MouseButton::Left) {
        ev_end_turn.send(EndTurnEvent(game_state.0));
        debug!("End Turn Event sent");
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
