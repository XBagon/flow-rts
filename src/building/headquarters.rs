use bevy::prelude::*;

use super::Building;

pub struct HeadQuartersPlugin;

impl Plugin for HeadQuartersPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, HeadQuartersController::setup)
            .add_event::<SpawnHeadQuarters>()
            .add_systems(Update, SpawnHeadQuarters::handle);
    }
}

#[derive(Resource)]
pub struct HeadQuartersController {
    pub scene: Handle<Scene>,
}

impl HeadQuartersController {
    pub fn setup(mut commands: Commands, asset_server: ResMut<AssetServer>) {
        let scene = asset_server.load("models\\towerRound_sampleF.glb#Scene0");

        commands.insert_resource(HeadQuartersController {
            scene,
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
        head_quaters_spawner: ResMut<HeadQuartersController>,
        mut spawn_head_quarters: EventReader<SpawnHeadQuarters>,
    ) {
        for event in spawn_head_quarters.read() {
            commands.spawn((
                HeadQuarters {},
                Building::default(),
                SceneBundle {
                    scene: head_quaters_spawner.scene.clone(),
                    transform: Transform::from_translation(event.position),
                    ..default()
                },
            ));
        }
    }
}

#[derive(Component)]
pub struct HeadQuarters {}
