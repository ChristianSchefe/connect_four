use std::time::Duration;

use bevy::{prelude::*, render::camera::ScalingMode, sprite::MaterialMesh2dBundle};
use bevy_tweening::lens::{ColorMaterialColorLens, TransformPositionLens, TransformScaleLens};

use crate::*;

pub const BACKGROUND_COLOR: Color = Color::rgb(1.0, 1.0, 1.0);
pub const TILE_COLOR: Color = Color::rgb(1.0, 1.0, 1.0);
pub const BOARD_COLOR: Color = Color::rgb(0.85, 0.85, 0.85);
pub const PLAYER1_COLOR: Color = Color::hsl(190.0, 0.9, 0.5);
pub const PLAYER2_COLOR: Color = Color::hsl(340.0, 0.9, 0.5);
pub const GOLD_COLOR: Color = Color::hsl(47.0, 0.9, 0.58);

#[derive(Component)]
pub struct WinLine;

#[derive(Component)]
pub struct Tile(Option<Player>, UVec2);

#[derive(Component)]
struct TurnIndicator(Option<Player>);

#[derive(Component)]
pub struct MainCamera;

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

pub struct VisualsPlugin;

impl Plugin for VisualsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(BACKGROUND_COLOR))
            .add_systems(Startup, (setup_camera, setup_ui, setup_board))
            .add_systems(Update, (update_turn_indicator, update_tiles, draw_line))
            .add_systems(
                Update,
                (
                    asset_animator_system::<ColorMaterial>.in_set(AnimationSystem::AnimationUpdate),
                    component_animator_system::<Transform>.in_set(AnimationSystem::AnimationUpdate),
                    component_animator_system::<BackgroundColor>.in_set(AnimationSystem::AnimationUpdate),
                ),
            );
    }
}

fn setup_camera(mut commands: Commands) {
    let mut cam = Camera2dBundle::default();
    cam.projection.scaling_mode = ScalingMode::AutoMin { min_width: 8.0, min_height: 8.0 };

    commands.spawn((cam, MainCamera));
}

fn setup_ui(mut commands: Commands) {
    commands.spawn((
        NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Px(30.0),
                ..default()
            },
            background_color: PLAYER1_COLOR.into(),
            ..default()
        },
        TurnIndicator(None),
    ));
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
    for y in 0..board.size.y {
        for x in 0..board.size.x {
            let pos = UVec2 { x, y };
            commands.spawn((
                Tile(None, pos),
                MaterialMesh2dBundle {
                    mesh: meshes.add(shape::Circle::default().into()).into(),
                    material: materials.add(ColorMaterial::from(Color::RED.with_a(0.0))),
                    transform: Transform {
                        translation: board.grid_to_world(pos).extend(0.0),
                        scale: Vec3::new(0.7, 0.7, 1.0),
                        ..default()
                    },
                    visibility: Visibility::Visible,
                    ..default()
                },
            ));
            commands.spawn((SpriteBundle {
                transform: Transform {
                    translation: board.grid_to_world(pos).extend(-1.0),
                    scale: Vec3::new(1.0 - tile_margin, 1.0 - tile_margin, 1.0),
                    ..default()
                },
                sprite: Sprite { color: TILE_COLOR, ..default() },
                ..default()
            },));
            commands.spawn((SpriteBundle {
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
            },));
        }
    }
}

fn update_turn_indicator(mut commands: Commands, mut query: Query<(Entity, &mut TurnIndicator, &BackgroundColor, Option<&mut Animator<BackgroundColor>>)>, board: Res<Board>) {
    if let Ok((entity, mut turn_indicator, background_color, maybe_animator)) = query.get_single_mut() {
        let new_state = match board.get_board_state() {
            BoardState::GameOver(_) => None,
            BoardState::Playing => Some(board.cur_player),
        };

        if turn_indicator.0 == new_state {
            return;
        }

        let start_color = background_color.0;
        let end_color = match new_state {
            None => GOLD_COLOR,
            Some(Player::PlayerOne) => PLAYER1_COLOR,
            Some(Player::PlayerTwo) => PLAYER2_COLOR,
        };

        let tween = Tween::new(
            EaseFunction::CubicOut,
            Duration::from_secs_f32(1.0),
            BackgroundColorLens {
                start: start_color,
                end: end_color,
            },
        );

        if let Some(mut animator) = maybe_animator {
            animator.set_tweenable(tween);
        } else {
            commands.entity(entity).insert(Animator::new(tween));
        }

        turn_indicator.0 = new_state;
    }
}

fn update_tiles(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Tile, &Handle<ColorMaterial>, Option<&mut AssetAnimator<ColorMaterial>>)>,
    board: Res<Board>,
    materials: Res<Assets<ColorMaterial>>,
) {
    for (entity, mut tile, sprite, maybe_animator) in query.iter_mut() {
        let new_state = board.get(tile.1);
        // info!("Update Tile at {}", ((*tile).1));

        if tile.0 == new_state {
            continue;
        }
        tile.0 = new_state;

        let original_color = materials.get(sprite).map_or(Color::BLACK, |x| x.color);

        let start_color = match new_state {
            None => original_color,
            Some(Player::PlayerOne) => PLAYER1_COLOR.with_a(0.0),
            Some(Player::PlayerTwo) => PLAYER2_COLOR.with_a(0.0),
        };
        let end_color = match new_state {
            None => original_color.with_a(0.0),
            Some(Player::PlayerOne) => PLAYER1_COLOR,
            Some(Player::PlayerTwo) => PLAYER2_COLOR,
        };

        let tween = Tween::new(
            EaseFunction::CubicOut,
            Duration::from_secs_f32(1.0),
            ColorMaterialColorLens {
                start: start_color,
                end: end_color,
            },
        );

        if let Some(mut animator) = maybe_animator {
            animator.set_tweenable(tween);
        } else {
            commands.entity(entity).insert(AssetAnimator::new(tween));
        }
    }
}

fn draw_line(mut commands: Commands, mut reader: EventReader<GameEvent>, board: Res<Board>) {
    for event in reader.read() {
        if let GameEvent::EndGame(GameResult::Win(player, line)) = event {
            warn!("{:?}", board.levels);

            let pos_diff = line.1.as_vec2() - line.0.as_vec2();
            let pos_tween = Tween::new(
                EaseFunction::CubicInOut,
                Duration::from_secs_f32(1.0),
                TransformPositionLens {
                    start: board.grid_to_world(line.0).extend(1.0),
                    end: board.vec2_to_world((line.0 + line.1).as_vec2() * 0.5).extend(1.0),
                },
            );

            let scale_tween = Tween::new(
                EaseFunction::CubicInOut,
                Duration::from_secs_f32(1.0),
                TransformScaleLens {
                    start: Vec3::new(0.0, 0.2, 1.0),
                    end: Vec3::new(pos_diff.length(), 0.2, 1.0),
                },
            );

            let vanish_pos_tween = Tween::new(
                EaseFunction::CubicInOut,
                Duration::from_secs_f32(1.0),
                TransformPositionLens {
                    start: board.vec2_to_world((line.0 + line.1).as_vec2() * 0.5).extend(1.0),
                    end: board.grid_to_world(line.1).extend(1.0),
                },
            );

            let vanish_scale_tween = Tween::new(
                EaseFunction::CubicInOut,
                Duration::from_secs_f32(1.0),
                TransformScaleLens {
                    start: Vec3::new(pos_diff.length(), 0.2, 1.0),
                    end: Vec3::new(0.0, 0.2, 1.0),
                },
            );

            let appear_tween = Tracks::new([pos_tween, scale_tween]);
            let vanish_tween = Tracks::new([vanish_pos_tween, vanish_scale_tween]);

            let tween: Sequence<Transform> = Delay::new(Duration::from_secs_f32(1.0))
                .then(appear_tween)
                .then(Delay::new(Duration::from_secs_f32(5.0)))
                .then(vanish_tween);

            let color = match player {
                Player::PlayerOne => PLAYER1_COLOR,
                Player::PlayerTwo => PLAYER2_COLOR,
            };

            commands.spawn((
                WinLine,
                Animator::new(tween),
                SpriteBundle {
                    transform: Transform {
                        translation: board.grid_to_world(line.0).extend(1.0),
                        scale: Vec3::new(0.0, 0.2, 1.0),
                        rotation: Quat::from_rotation_z(Vec2::new(1.0, 0.0).angle_between(pos_diff)),
                    },
                    sprite: Sprite { color, ..default() },
                    ..default()
                },
            ));
        }
    }
}
