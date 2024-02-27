use bevy::prelude::*;
use bevy_xpbd_3d::prelude::*;

use crate::{
    building::{headquarters::SpawnHeadQuarters, tree::SpawnTree, BuildingPlugins},
    input::InputPlugin,
    unit::UnitPlugin,
    way::WayPlugin,
};

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, Game::setup).add_plugins((
            PhysicsPlugins::default(),
            //PhysicsDebugPlugin::default(),
            InputPlugin,
            WayPlugin,
            BuildingPlugins.build(),
            UnitPlugin,
        ));
    }
}

#[derive(Resource)]
pub struct Game {}

impl Game {
    pub fn setup(
        mut commands: Commands,
        mut ev_spawn_head_quarters: EventWriter<SpawnHeadQuarters>,
        mut ev_spawn_tree: EventWriter<SpawnTree>,
    ) {
        commands.insert_resource(Game {});
        ev_spawn_head_quarters.send(SpawnHeadQuarters {
            position: Vec3::new(0.0, 0.0, 5.0),
        });
        ev_spawn_tree.send(SpawnTree {
            position: Vec3::new(5.0, 0.0, 0.0),
        });
        ev_spawn_tree.send(SpawnTree {
            position: Vec3::new(-9.0, 0.0, -14.0),
        });
        ev_spawn_tree.send(SpawnTree {
            position: Vec3::new(-6.0, 0.0, -8.0),
        });
    }

    pub fn update(mut commands: Commands, time: Res<Time>, mut game: ResMut<Game>) {}
}
