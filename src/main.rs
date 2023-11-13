mod components;

use bevy::{
    prelude::*, render::camera::ScalingMode, sprite::MaterialMesh2dBundle, window::PrimaryWindow,
};
use bevy_tweening::{
    asset_animator_system, lens::ColorMaterialColorLens, AnimationSystem, AssetAnimator,
    EaseFunction, Tween, TweeningPlugin,
};
use components::{Board, GameStateMachine};

use crate::components::Player;

const BACKGROUND_COLOR: Color = Color::rgb(0.9, 0.9, 0.9);
const PLAYER1_COLOR: Color = Color::hsl(340.0, 0.9, 0.5);
const PLAYER2_COLOR: Color = Color::hsl(190.0, 0.9, 0.5);
const HOVER1_COLOR: Color = Color::hsl(190.0, 0.5, 0.9);
const HOVER2_COLOR: Color = Color::hsl(340.0, 0.5, 0.9);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameState {
    PlayerOneTurn,
    PlayerTwoTurn,
    GameOver,
}

#[derive(Event)]
struct EndTurnEvent(pub GameState, pub UVec2);

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
            asset_animator_system::<ColorMaterial>.in_set(AnimationSystem::AnimationUpdate),
        )
        .add_systems(Startup, (setup_camera, setup_board))
        .add_systems(Update, bevy::window::close_on_esc)
        .add_systems(
            Update,
            (do_move, end_turn, calc_world_mouse, update_tile, hover_tile),
        )
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
    // cam.projection.scaling_mode = ScalingMode::FixedVertical(10f32);
    cam.projection.scaling_mode = ScalingMode::AutoMin {
        min_width: 10.0,
        min_height: 9.0,
    };

    commands.spawn((cam, MainCamera));
}

fn setup_board(
    mut commands: Commands,
    board: Res<Board>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for y in 0..board.size.y {
        for x in 0..board.size.x {
            let pos = UVec2 { x, y };
            commands.spawn((
                GridPosition(pos),
                AnimationTarget(None),
                MaterialMesh2dBundle {
                    mesh: meshes.add(shape::Circle::default().into()).into(),
                    material: materials.add(ColorMaterial::from(Color::WHITE)),
                    transform: Transform {
                        translation: board.grid_to_world(pos).extend(0.0),
                        scale: Vec3::new(0.7, 0.7, 1.0),
                        ..default()
                    },
                    visibility: Visibility::Hidden,
                    ..default()
                },
            ));
            commands.spawn((
                GridPosition(pos),
                SpriteBundle {
                    transform: Transform {
                        translation: board.grid_to_world(pos).extend(-1.0),
                        scale: Vec3::new(0.9, 0.9, 1.0),
                        ..default()
                    },
                    sprite: Sprite {
                        color: Color::WHITE,
                        ..default()
                    },
                    ..default()
                },
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

fn hover_tile(
    mouse_position: Res<WorldCoords>,
    game_state: Res<GameStateMachine>,
    board: ResMut<Board>,
    mut tiles: Query<(&GridPosition, &mut Sprite)>,
) {
    let grid_pos = board.world_to_grid(mouse_position.0);
    for (pos, mut sprite) in tiles.iter_mut() {
        let hover_color = if grid_pos.is_some_and(|p| p == pos.0) {
            match game_state.0 {
                GameState::PlayerOneTurn => HOVER1_COLOR,
                GameState::PlayerTwoTurn => HOVER2_COLOR,
                GameState::GameOver => Color::WHITE,
            }
        } else {
            Color::WHITE
        };
        sprite.color = hover_color;
    }
}

fn do_move(
    input: Res<Input<MouseButton>>,
    game_state: Res<GameStateMachine>,
    mut board: ResMut<Board>,
    mouse_position: Res<WorldCoords>,
    mut ev_end_turn: EventWriter<EndTurnEvent>,
) {
    if game_state.0 == GameState::GameOver {
        return;
    }
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
                ev_end_turn.send(EndTurnEvent(game_state.0, grid_pos));
            } else {
                info!("Can't place here");
            }
        }
    }
}

fn check_for_win(board: &Board, team: Player, updated_pos: UVec2) -> bool {
    let check_dir = |dir: IVec2| {
        let mut has_four = true;
        for i in 0..4 {
            let pos = updated_pos.as_ivec2() + dir * i;

            if !board.valid_ivec_pos(pos) {
                has_four = false;
                break;
            }
            let tile = board.get(pos.as_uvec2());
            if tile.is_none() || tile.is_some_and(|p| p != team) {
                has_four = false;
                break;
            }
        }
        if has_four {
            return true;
        }
        false
    };

    check_dir(IVec2 { x: 1, y: 0 })
        || check_dir(IVec2 { x: 1, y: 1 })
        || check_dir(IVec2 { x: 0, y: 1 })
        || check_dir(IVec2 { x: -1, y: 1 })
        || check_dir(IVec2 { x: -1, y: 0 })
        || check_dir(IVec2 { x: -1, y: -1 })
        || check_dir(IVec2 { x: 0, y: -1 })
        || check_dir(IVec2 { x: 1, y: -1 })
}

fn update_tile(
    mut commands: Commands,
    mut tiles: Query<(
        &GridPosition,
        &Handle<ColorMaterial>,
        &mut AnimationTarget,
        &mut Visibility,
        Entity,
    )>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    board: Res<Board>,
) {
    for (pos, sprite, mut animation_target, mut visibility, entity) in tiles.iter_mut() {
        let tile_type = board.get(pos.0);
        let end_color = match tile_type {
            Some(Player::PlayerOne) => PLAYER1_COLOR,
            Some(Player::PlayerTwo) => PLAYER2_COLOR,
            None => Color::WHITE,
        };
        *visibility = if tile_type.is_none() {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        };
        if tile_type == animation_target.0 {
            continue;
        }
        animation_target.0 = tile_type;

        if let Some(col) = materials.get_mut(sprite) {
            col.color = end_color.with_a(0.0);
        }

        let tween = Tween::new(
            EaseFunction::CubicOut,
            std::time::Duration::from_secs_f32(1.5),
            ColorMaterialColorLens {
                start: end_color.with_a(0.0),
                end: end_color,
            },
        );
        info!("Add animator at {:?}", pos);
        commands
            .entity(entity)
            .remove::<AssetAnimator<ColorMaterial>>()
            .insert(AssetAnimator::new(tween));
    }
}

fn end_turn(
    mut ev_end_turn: EventReader<EndTurnEvent>,
    mut game_state: ResMut<GameStateMachine>,
    board: Res<Board>,
) {
    for ev in ev_end_turn.read() {
        if ev.0 != game_state.0 {
            warn!("End turn from wrong Player!");
            continue;
        }

        let cur_player = match &game_state.0 {
            GameState::PlayerOneTurn => Some(Player::PlayerOne),
            GameState::PlayerTwoTurn => Some(Player::PlayerTwo),
            GameState::GameOver => None,
        };

        if let Some(p) = cur_player {
            let win = check_for_win(board.as_ref(), p, ev.1);
            if win {
                info!("Player {:?} has won", p);
                game_state.0 = GameState::GameOver;
            } else {
                info!("No win");
            }
        }

        game_state.0 = match &game_state.0 {
            GameState::PlayerOneTurn => GameState::PlayerTwoTurn,
            GameState::PlayerTwoTurn => GameState::PlayerOneTurn,
            GameState::GameOver => GameState::GameOver,
        };

        info!(
            "End Turn from {:?} at {}. New State: {:?}",
            ev.0, ev.1, game_state.0
        );
    }
}
