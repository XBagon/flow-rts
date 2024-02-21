use bevy::prelude::*;

pub struct AssetPlugin;

impl Plugin for AssetPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, AssetController::setup);
    }
}

#[derive(Resource)]
pub struct AssetController {
    pub head_quarters_scene: Handle<Scene>,
}

impl AssetController {
    pub fn setup(mut commands: Commands, asset_server: ResMut<AssetServer>) {
        let scene = asset_server.load("models\\towerRound_sampleF.glb#Scene0");

        commands.insert_resource(AssetController {
            head_quarters_scene: scene,
        });
    }
}
