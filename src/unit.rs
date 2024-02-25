use bevy::prelude::*;

use crate::building::Building;
pub struct UnitPlugin;

impl Plugin for UnitPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnUnit>()
            .add_systems(Startup, UnitSpawner::setup)
            .add_systems(PreUpdate, SpawnUnit::handle)
            .add_systems(Update, Unit::update);
    }
}

#[derive(Resource)]
pub struct UnitSpawner {
    pub scene: Handle<Scene>,
}

impl UnitSpawner {
    pub fn setup(mut commands: Commands, asset_server: ResMut<AssetServer>) {
        let scene = asset_server.load("models\\unit.glb#Scene0");

        commands.insert_resource(UnitSpawner {
            scene,
        });
    }
}

#[derive(Event, Debug)]
pub struct SpawnUnit {
    pub unit: Unit,
}

impl SpawnUnit {
    pub fn handle(
        mut commands: Commands,
        unit_spawner: Res<UnitSpawner>,
        mut spawn_unit: EventReader<SpawnUnit>,
        q_buildings: Query<&Transform, With<Building>>,
    ) {
        for event in spawn_unit.read() {
            info!(target: "events", "{:?}", event);
            let from_building = q_buildings.get(event.unit.from_building).unwrap();
            let to_building = q_buildings.get(event.unit.to_building).unwrap();
            let direction = to_building.translation - from_building.translation;
            commands.spawn((
                event.unit.clone(),
                SceneBundle {
                    scene: unit_spawner.scene.clone(),
                    transform: Transform {
                        translation: from_building.translation,
                        scale: Vec3::splat(0.2),
                        ..default()
                    }
                    .looking_to(direction, Vec3::Y),
                    ..default()
                },
            ));
        }
    }
}

#[derive(Component, Clone, Debug)]
pub struct Unit {
    pub from_building: Entity,
    pub to_building: Entity,
}

impl Unit {
    pub fn update(
        mut commands: Commands,
        time: Res<Time>,
        mut q_units: Query<(Entity, &Unit, &mut Transform), Without<Building>>,
        q_buildings: Query<&Transform, With<Building>>,
    ) {
        for (entity, unit, mut transform) in q_units.iter_mut() {
            let to_building = q_buildings.get(unit.to_building).unwrap();
            let direction = to_building.translation - transform.translation;
            let distance = direction.length();
            let speed = 1.0;
            let direction = direction.normalize();
            let movement = direction * speed * time.delta_seconds();
            if distance > movement.length() {
                transform.translation += movement;
            } else {
                commands.entity(entity).despawn();
            }
        }
    }
}
