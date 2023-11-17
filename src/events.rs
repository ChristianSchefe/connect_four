use bevy::prelude::*;

use crate::{
    board::{GameResult, Move},
    player::Player,
};

pub struct EventBusPlugin;

impl Plugin for EventBusPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<GameEvent>()
            .add_event::<DelayEvent>()
            .add_systems(Update, (handle_delay_event, handle_delay_event_timer));
    }
}

fn handle_delay_event(mut commands: Commands, mut reader: EventReader<DelayEvent>) {
    for event in reader.read() {
        commands.spawn(DelayEventTimer(event.0, Timer::from_seconds(event.1, TimerMode::Once)));
    }
}

fn handle_delay_event_timer(mut commands: Commands, mut query: Query<(Entity, &mut DelayEventTimer)>, mut writer: EventWriter<GameEvent>, time: Res<Time>) {
    for (entity, mut timer) in query.iter_mut() {
        timer.1.tick(time.delta());

        if timer.1.just_finished() {
            writer.send(timer.0);
            commands.entity(entity).despawn();
        }
    }
}

#[derive(Event, Clone, Copy, Debug)]
pub enum GameEvent {
    StartGame(Player),
    RequestMove(Player),
    DoMove(Move),
    EndGame(GameResult),
    ResetBoard,
}

#[derive(Event)]
pub struct DelayEvent(pub GameEvent, pub f32);

#[derive(Component)]
struct DelayEventTimer(GameEvent, Timer);
