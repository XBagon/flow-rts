//! RTS game you play by controlling the flow of units between buildings

use assets::AssetPlugin;
use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
    render::{
        camera::ScalingMode,
        settings::{Backends, WgpuSettings},
        RenderPlugin,
    },
};
use game::GamePlugin;

mod assets;
mod building;
mod game;
mod input;
mod way;

fn main() {
    let mut app = App::new();
    app.add_plugins((
        DefaultPlugins
            .set(RenderPlugin {
                render_creation: WgpuSettings {
                    backends: Some(Backends::VULKAN),
                    ..default()
                }
                .into(),
                ..default()
            })
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "FlowRTS".into(),
                    ..default()
                }),
                ..default()
            }),
        FrameTimeDiagnosticsPlugin::default(),
        LogDiagnosticsPlugin::default(),
        AssetPlugin,
        GamePlugin,
    ))
    .insert_resource(Msaa::Sample4)
    .add_systems(Startup, setup)
    .add_systems(Update, debug);

    app.run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(10.0, 10.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
        projection: Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::WindowSize(40.0),
            ..default()
        }),
        ..default()
    });

    commands.spawn(DirectionalLightBundle {
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });
}

fn debug(world: &mut World) {
    //for entity in world.query::<Entity>().iter(world) {
    //    //print out children of the entity
    //    for child in world
    //        .get::<Children>(entity)
    //        .map(|children| children.iter().collect::<Vec<_>>())
    //        .unwrap_or_default()
    //    {
    //        println!("{:?} is child of {:?}", child, entity);
    //    }
    //}
}
