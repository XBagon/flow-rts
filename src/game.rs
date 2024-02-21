use bevy::prelude::*;

use crate::{
    building::{headquarters::SpawnHeadQuarters, BuildingPlugins},
    input::InputPlugin,
    way::WayPlugin,
};

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, Game::setup).add_plugins((
            InputPlugin,
            WayPlugin,
            BuildingPlugins.build(),
        ));
    }
}

#[derive(Resource)]
pub struct Game {}

impl Game {
    pub fn setup(
        mut commands: Commands,
        mut spawn_head_quarters_ev: EventWriter<SpawnHeadQuarters>,
    ) {
        commands.insert_resource(Game {});
        spawn_head_quarters_ev.send(SpawnHeadQuarters {
            position: Vec3::new(0.0, 0.0, 0.0),
        });
    }

    pub fn update(mut commands: Commands, time: Res<Time>, mut game: ResMut<Game>) {}
}
