use bevy::prelude::*;
use bevy_xpbd_3d::plugins::collision::{Collider, ColliderAabb};

use crate::unit::{SpawnUnit, Unit};

use super::Building;

pub struct HeadQuartersPlugin;

impl Plugin for HeadQuartersPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<HeadQuartersSpawner>()
            .add_systems(Startup, HeadQuartersSpawner::setup)
            .add_event::<SpawnHeadQuarters>()
            .add_systems(Update, (SpawnHeadQuarters::handle, HeadQuarters::update));
    }
}

#[derive(Resource, Reflect)]
pub struct HeadQuartersSpawner {
    pub scene: Handle<Scene>,
    pub default_cooldown: f32,
}

impl HeadQuartersSpawner {
    pub fn setup(mut commands: Commands, asset_server: ResMut<AssetServer>) {
        let scene = asset_server.load("models\\towerRound_sampleF.glb#Scene0");

        commands.insert_resource(HeadQuartersSpawner {
            scene,
            default_cooldown: 5.0,
        });
    }
}

#[derive(Event)]
pub struct SpawnHeadQuarters {
    pub position: Vec3,
}

impl SpawnHeadQuarters {
    pub fn handle(
        mut commands: Commands,
        head_quaters_spawner: ResMut<HeadQuartersSpawner>,
        mut spawn_head_quarters: EventReader<SpawnHeadQuarters>,
    ) {
        for event in spawn_head_quarters.read() {
            commands.spawn((
                HeadQuarters {
                    spawn_timer: Timer::from_seconds(
                        head_quaters_spawner.default_cooldown,
                        TimerMode::Repeating,
                    ),
                    cursor: 0,
                },
                Building::default(),
                SceneBundle {
                    scene: head_quaters_spawner.scene.clone(),
                    transform: Transform::from_translation(event.position),
                    ..default()
                },
                Collider::cuboid(1.0, 1.0, 1.0),
            ));
        }
    }
}

#[derive(Component)]
pub struct HeadQuarters {
    pub spawn_timer: Timer,
    cursor: usize,
}

impl HeadQuarters {
    pub fn update(
        time: Res<Time>,
        mut head_quarters: Query<(Entity, &mut HeadQuarters, &Building, &Transform)>,
        mut ev_spawn_unit: EventWriter<SpawnUnit>,
    ) {
        for (entity, mut head_quarters, building, transform) in head_quarters.iter_mut() {
            head_quarters.spawn_timer.tick(time.delta());
            if building.connected.is_empty() {
                head_quarters.spawn_timer.set_mode(TimerMode::Once);
            } else {
                if head_quarters.spawn_timer.finished() {
                    dbg!("spawn");
                    for _ in 0..head_quarters.spawn_timer.times_finished_this_tick().max(1) {
                        head_quarters.cursor %= building.connected.len();
                        let unit = Unit {
                            from_building: entity,
                            to_building: building.connected[head_quarters.cursor],
                        };
                        head_quarters.cursor += 1;
                        ev_spawn_unit.send(SpawnUnit {
                            unit,
                        });
                        head_quarters.spawn_timer.reset();
                    }
                }
                head_quarters.spawn_timer.set_mode(TimerMode::Repeating);
            }
        }
    }
}
