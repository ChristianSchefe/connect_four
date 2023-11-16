mod ai;
mod components;

use std::time::Duration;

use ai::{Ai, AiPlugin};
use bevy::{prelude::*, render::camera::ScalingMode, sprite::MaterialMesh2dBundle, window::PrimaryWindow};
use bevy_tweening::{
    asset_animator_system, component_animator_system,
    lens::{ColorMaterialColorLens, TransformPositionLens, TransformScaleLens},
    AnimationSystem, Animator, AssetAnimator, Delay, EaseFunction, Tracks, Tween, TweeningPlugin,
};
use components::*;

const BACKGROUND_COLOR: Color = Color::rgb(1.0, 1.0, 1.0);
const TILE_COLOR: Color = Color::rgb(1.0, 1.0, 1.0);
const BOARD_COLOR: Color = Color::rgb(0.85, 0.85, 0.85);
const PLAYER1_COLOR: Color = Color::hsl(190.0, 0.9, 0.5);
const PLAYER2_COLOR: Color = Color::hsl(340.0, 0.9, 0.5);
const GOLD_COLOR: Color = Color::hsl(47.0, 0.9, 0.58);
const WIN_DIRECTIONS: [IVec2; 4] = [
    IVec2::new(1, 0),
    IVec2::new(1, 1),
    IVec2::new(0, 1),
    IVec2::new(-1, 1),
    // IVec2::new(-1, 0),
    // IVec2::new(-1, -1),
    // IVec2::new(0, -1),
    // IVec2::new(1, -1),
];

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, TweeningPlugin, AiPlugin))
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .insert_resource(Board::new())
        .add_systems(
            Update,
            (
                asset_animator_system::<ColorMaterial>.in_set(AnimationSystem::AnimationUpdate),
                component_animator_system::<Transform>.in_set(AnimationSystem::AnimationUpdate),
                component_animator_system::<BackgroundColor>.in_set(AnimationSystem::AnimationUpdate),
            ),
        )
        .add_systems(Startup, (setup_camera, setup_board, setup_ui))
        .add_systems(Update, bevy::window::close_on_esc)
        .add_systems(Update, (do_move, end_turn, calc_world_mouse, update_tile, hover_tile, spawn_win_line))
        .insert_resource(GameStateMachine(GameState::PlayerOneTurn))
        .init_resource::<WorldCoords>()
        .add_event::<EndTurnEvent>()
        .add_event::<WinEvent>()
        .add_event::<RequestMoveEvent>()
        .run();
}

fn setup_camera(mut commands: Commands) {
    let mut cam = Camera2dBundle::default();
    cam.projection.scaling_mode = ScalingMode::AutoMin { min_width: 8.0, min_height: 8.0 };

    commands.spawn((cam, MainCamera));
}

fn setup_ui(mut commands: Commands) {
    commands.spawn((NodeBundle {
        style: Style {
            width: Val::Percent(100.0),
            height: Val::Px(30.0),
            ..default()
        },
        background_color: PLAYER1_COLOR.into(),
        ..default()
    },));
}

fn setup_board(mut commands: Commands, board: Res<Board>, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<ColorMaterial>>) {
    let tile_margin = 0.025;
    commands.spawn((SpriteBundle {
        transform: Transform {
            translation: Vec2::new(0.0, 0.0).extend(-5.0),
            scale: Vec3::new(board.size.x as f32 + tile_margin, board.size.y as f32 + tile_margin, 1.0),
            ..default()
        },
        sprite: Sprite { color: BOARD_COLOR, ..default() },
        ..default()
    },));
    commands.spawn((Ai { player: Player::PlayerTwo },));
    commands.spawn((Ai { player: Player::PlayerOne },));
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
                        scale: Vec3::new(1.0 - tile_margin, 1.0 - tile_margin, 1.0),
                        ..default()
                    },
                    sprite: Sprite { color: TILE_COLOR, ..default() },
                    ..default()
                },
            ));
            commands.spawn((
                GridPosition(pos),
                SpriteBundle {
                    transform: Transform {
                        translation: board.grid_to_world(pos).extend(-2.0),
                        scale: Vec3::new(1.0 + tile_margin, 1.0 + tile_margin, 1.0),
                        ..default()
                    },
                    sprite: Sprite {
                        color: Color::rgb(0.7, 0.7, 0.7),
                        ..default()
                    },
                    visibility: Visibility::Hidden,
                    ..default()
                },
                TileMarker,
            ));
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

fn hover_tile(mouse_position: Res<WorldCoords>, board: ResMut<Board>, mut tiles: Query<(&GridPosition, &mut Visibility), With<TileMarker>>) {
    let grid_pos = board.world_to_grid(mouse_position.0);
    for (pos, mut visibility) in tiles.iter_mut() {
        *visibility = if grid_pos.is_some_and(|p| p == pos.0) {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
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
        debug!("World coords: {}/{}", mouse_position.0.x, mouse_position.0.y);
        if let Some(grid_pos) = board.world_to_grid(mouse_position.0) {
            debug!("Grid pos: {}/{}", grid_pos.x, grid_pos.y);
            debug!("End Turn Event sent");

            if let Some(cur_player) = match &game_state.0 {
                GameState::PlayerOneTurn => Some(Player::PlayerOne),
                GameState::PlayerTwoTurn => Some(Player::PlayerTwo),
                GameState::GameOver => None,
            } {
                if board.get(grid_pos).is_none() && (grid_pos.y == 0 || board.get(grid_pos - UVec2::new(0, 1)).is_some()) {
                    board.do_move(Move {
                        pos: grid_pos,
                        player: cur_player,
                    });
                    ev_end_turn.send(EndTurnEvent(game_state.0, grid_pos));
                } else {
                    info!("Can't place here");
                }
            }
        }
    }
}

fn update_tile(
    mut commands: Commands,
    mut tiles: Query<(
        &GridPosition,
        &Handle<ColorMaterial>,
        &mut AnimationTarget,
        &mut Visibility,
        Option<&mut AssetAnimator<ColorMaterial>>,
        Entity,
    )>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    board: Res<Board>,
) {
    for (pos, sprite, mut animation_target, mut visibility, maybe_animator, entity) in tiles.iter_mut() {
        let tile_type = board.get(pos.0);
        let end_color = match tile_type {
            Some(Player::PlayerOne) => PLAYER1_COLOR,
            Some(Player::PlayerTwo) => PLAYER2_COLOR,
            None => Color::WHITE,
        };
        *visibility = if tile_type.is_none() { Visibility::Hidden } else { Visibility::Inherited };
        if tile_type == animation_target.0 {
            continue;
        }
        animation_target.0 = tile_type;

        if let Some(col) = materials.get_mut(sprite) {
            col.color = end_color.with_a(0.0);
        }

        let tween = Tween::new(
            EaseFunction::CubicOut,
            std::time::Duration::from_secs_f32(1.0),
            ColorMaterialColorLens {
                start: end_color.with_a(0.0),
                end: end_color,
            },
        );
        debug!("Add animator at {:?}", pos);

        if let Some(mut animator) = maybe_animator {
            animator.set_tweenable(tween);
        } else {
            commands.entity(entity).insert(AssetAnimator::new(tween));
        }
    }
}

fn end_turn(
    mut commands: Commands,
    mut ev_end_turn: EventReader<EndTurnEvent>,
    mut game_state: ResMut<GameStateMachine>,
    board: Res<Board>,
    mut win_event_writer: EventWriter<WinEvent>,
    mut request_move_event: EventWriter<RequestMoveEvent>,
    mut turn_indicator: Query<(&mut BackgroundColor, Option<&mut Animator<BackgroundColor>>, Entity)>,
) {
    for ev in ev_end_turn.read() {
        if ev.0 != game_state.0 {
            warn!("End turn from wrong Player!");
            continue;
        }

        if board.is_draw() {
            info!("Draw!");
            game_state.0 = GameState::GameOver;
        }

        let cur_player = match game_state.0 {
            GameState::PlayerOneTurn => Some(Player::PlayerOne),
            GameState::PlayerTwoTurn => Some(Player::PlayerTwo),
            GameState::GameOver => None,
        };

        if let Some(p) = cur_player {
            info!("Check if {:?} has won", p);
            if let Some((from_pos, to_pos)) = board.check_for_win(p, ev.1) {
                info!("{:?} has won", p);
                game_state.0 = GameState::GameOver;
                win_event_writer.send(WinEvent {
                    winning_player: p,
                    from_pos,
                    to_pos,
                });
            } else {
                debug!("No win");
            }
        }

        game_state.0 = match game_state.0 {
            GameState::PlayerOneTurn => GameState::PlayerTwoTurn,
            GameState::PlayerTwoTurn => GameState::PlayerOneTurn,
            GameState::GameOver => GameState::GameOver,
        };

        let (turn_indicator_entity, maybe_animator, entity) = turn_indicator.single_mut();

        let tween = Tween::new(
            EaseFunction::CubicOut,
            std::time::Duration::from_secs_f32(1.0),
            BackgroundColorLens {
                start: turn_indicator_entity.0,
                end: match game_state.0 {
                    GameState::PlayerOneTurn => PLAYER1_COLOR,
                    GameState::PlayerTwoTurn => PLAYER2_COLOR,
                    GameState::GameOver => GOLD_COLOR,
                },
            },
        );

        if let Some(mut animator) = maybe_animator {
            animator.set_tweenable(tween);
        } else {
            commands.entity(entity).insert(Animator::new(tween));
        }

        info!("End Turn from {:?} at {}. New State: {:?}", ev.0, ev.1, game_state.0);

        if let Some(next_player) = match game_state.0 {
            GameState::PlayerOneTurn => Some(Player::PlayerOne),
            GameState::PlayerTwoTurn => Some(Player::PlayerTwo),
            GameState::GameOver => None,
        } {
            info!("Request move from {:?}", next_player);
            request_move_event.send(RequestMoveEvent(next_player));
        }
    }
}

fn spawn_win_line(mut commands: Commands, mut win_event: EventReader<WinEvent>, board: Res<Board>) {
    for ev in win_event.read() {
        info!("Win event! {:?}", ev);
        let center_pos = (ev.from_pos.as_vec2() + ev.to_pos.as_vec2()) * 0.5;
        let diff_vec = ev.from_pos.as_vec2() - ev.to_pos.as_vec2();

        let start_scale = Vec3::new(0.0, 0.2, 1.0);

        let track = Tracks::<Transform>::new([
            Tween::new(
                EaseFunction::CubicInOut,
                Duration::from_secs_f32(1.0),
                TransformScaleLens {
                    start: Vec3::new(0.0, 0.2, 1.0),
                    end: Vec3::new(diff_vec.length(), 0.2, 1.0),
                },
            ),
            Tween::new(
                EaseFunction::CubicInOut,
                Duration::from_secs_f32(1.0),
                TransformPositionLens {
                    start: board.vec2_to_world(ev.from_pos.as_vec2()).extend(1.0),
                    end: board.vec2_to_world(center_pos).extend(1.0),
                },
            ),
        ]);
        let animation = Delay::new(Duration::from_secs_f32(0.5)).then(track);

        commands.spawn((
            Animator::new(animation),
            SpriteBundle {
                transform: Transform {
                    translation: board.vec2_to_world(center_pos).extend(1.0),
                    scale: start_scale,
                    rotation: Quat::from_rotation_z(Vec2::angle_between(Vec2::new(1.0, 0.0), diff_vec)),
                },
                sprite: Sprite {
                    color: match ev.winning_player {
                        Player::PlayerOne => PLAYER1_COLOR,
                        Player::PlayerTwo => PLAYER2_COLOR,
                    },
                    ..default()
                },
                ..default()
            },
        ));
    }
}
