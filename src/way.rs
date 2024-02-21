use bevy::{
    prelude::*,
    render::{
        mesh::{Indices, VertexAttributeValues},
        render_asset::RenderAssetUsages,
        render_resource::PrimitiveTopology,
    },
};

use crate::input::InputController;

pub struct WayPlugin;

impl Plugin for WayPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, WayController::setup)
            .add_event::<StartWay>()
            .add_event::<FinishWay>()
            .add_systems(Update, (StartWay::handle, FinishWay::handle, PlacingWay::update));
    }
}

#[derive(Resource)]
pub struct WayController {
    pub material: Handle<StandardMaterial>,
    pub mesh: Handle<Mesh>,
    pub currenty_placing: bool,
}

impl WayController {
    pub fn setup(
        mut commands: Commands,
        mut materials: ResMut<Assets<StandardMaterial>>,
        mut meshes: ResMut<Assets<Mesh>>,
    ) {
        let material = materials.add(Color::rgb(0.3, 0.5, 0.3));
        let base_mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
            .with_inserted_attribute(
                Mesh::ATTRIBUTE_POSITION,
                //vec![[-0.5, 0.0, -0.5], [0.5, 0.0, -0.5], [0.5, 0.0, 0.5], [-0.5, 0.0, 0.5]],
                vec![[0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0]],
            )
            .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0.0, 0.0, 1.0]; 4])
            .with_inserted_attribute(
                Mesh::ATTRIBUTE_UV_0,
                vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]],
            )
            .with_inserted_indices(Indices::U32(vec![0, 3, 1, 1, 3, 2]));
        let mesh = meshes.add(base_mesh);

        commands.insert_resource(WayController {
            material: material.clone(),
            mesh: mesh.clone(),
            currenty_placing: false,
        });
    }
}

#[derive(Component)]
pub struct PlacingWay {
    from: Vec3,
}

impl PlacingWay {
    pub fn update(
        mut meshes: ResMut<Assets<Mesh>>,
        mut query: Query<(&PlacingWay, &mut Handle<Mesh>)>,
        input_controller: Res<InputController>,
    ) {
        for (way, mesh) in query.iter_mut() {
            let vertex_positions =
                meshes.get_mut(mesh.id()).unwrap().attribute_mut(Mesh::ATTRIBUTE_POSITION).unwrap();
            let vertex_positions =
                if let VertexAttributeValues::Float32x3(vertex_positions) = vertex_positions {
                    vertex_positions
                } else {
                    return;
                };

            let Some(global_cursor) = input_controller.plane_position else {
                return;
            };

            let start_point = Vec3::ZERO;
            let end_point = global_cursor - way.from;
            let offset =
                (end_point - start_point).try_normalize().unwrap_or(Vec3::X).cross(Vec3::Y) * 0.5;

            vertex_positions[0] = (start_point + offset).to_array();
            vertex_positions[1] = (start_point - offset).to_array();
            vertex_positions[2] = (end_point - offset).to_array();
            vertex_positions[3] = (end_point + offset).to_array();
        }
    }
}

#[derive(Event)]
pub struct StartWay {
    pub from: Vec3,
}

impl StartWay {
    pub fn handle(
        mut commands: Commands,
        mut events: EventReader<StartWay>,
        mut controller: ResMut<WayController>,
        mut meshes: ResMut<Assets<Mesh>>,
    ) {
        if controller.currenty_placing {
            return;
        }
        if let Some(event) = events.read().next() {
            controller.currenty_placing = true;

            let base_mesh = meshes.get(controller.mesh.id()).unwrap().clone();
            let mesh = meshes.add(base_mesh);

            commands.spawn((
                PlacingWay {
                    from: event.from,
                },
                PbrBundle {
                    transform: Transform::from_translation(event.from)
                        .with_rotation(Quat::from_rotation_x(0.0)),
                    mesh,
                    material: controller.material.clone(),
                    ..default()
                },
            ));
        }
    }
}

#[derive(Event)]
pub struct FinishWay {
    pub abort: bool,
}

impl FinishWay {
    pub fn handle(
        mut commands: Commands,
        mut events: EventReader<FinishWay>,
        mut controller: ResMut<WayController>,
        query: Query<Entity, With<PlacingWay>>,
    ) {
        if !controller.currenty_placing {
            return;
        }
        if let Some(event) = events.read().next() {
            controller.currenty_placing = false;
            let entity = query.single();
            if event.abort {
                commands.entity(entity).despawn();
            } else {
                commands.entity(entity).remove::<PlacingWay>();
            }
        }
    }
}
