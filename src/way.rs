use bevy::{
    prelude::*,
    render::{
        mesh::{Indices, VertexAttributeValues},
        render_asset::RenderAssetUsages,
        render_resource::PrimitiveTopology,
    },
};

use crate::{
    building::Building,
    input::{InputController, InputEvent},
};

pub struct WayPlugin;

impl Plugin for WayPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, WayController::setup).add_event::<InteractWay>().add_systems(
            Update,
            ((WayController::handle_input, InteractWay::handle).chain(), PlacingWay::update),
        );
    }
}

#[derive(Resource)]
pub struct WayController {
    pub material: Handle<StandardMaterial>,
    pub mesh: Handle<Mesh>,
    pub start_building: Option<Entity>,
    pub connected: Vec<(Entity, Entity)>,
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
            start_building: None,
            connected: Vec::new(),
        });
    }

    fn handle_input(
        mut ev_input: EventReader<InputEvent>,
        mut ev_interact_way: EventWriter<InteractWay>,
        controller: Res<WayController>,
    ) {
        for event in ev_input.read() {
            match *event {
                InputEvent::ClickedOnBuilding {
                    building,
                } => {
                    if let Some(start_building) = controller.start_building {
                        if start_building == building {
                            continue;
                        }
                        ev_interact_way.send(InteractWay::Finish {
                            from: start_building,
                            connect_to: building,
                        });
                    } else {
                        ev_interact_way.send(InteractWay::Start {
                            from: building,
                        });
                    }
                }
                InputEvent::Abort => {
                    if let Some(start_building) = controller.start_building {
                        ev_interact_way.send(InteractWay::Abort {
                            aborted: start_building,
                        });
                    }
                }
                _ => {}
            }
        }
    }
}

#[derive(Component)]
pub struct PlacingWay {
    from: Entity,
}

impl PlacingWay {
    pub fn update(
        mut meshes: ResMut<Assets<Mesh>>,
        mut query: Query<(&PlacingWay, &mut Handle<Mesh>)>,
        q_buildings: Query<&Transform, With<Building>>,
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

            let global_end_point = if let Some(hovering_building) =
                input_controller.hovering_building
            //.and_then(|hovering_building|
            {
                //(hovering_building != way.from).then_some(hovering_building)
                //}) {
                let transform = q_buildings.get(hovering_building).unwrap();
                transform.translation
            } else if let Some(plane_position) = input_controller.plane_position {
                plane_position
            } else {
                return;
            };

            let start_point = Vec3::ZERO;
            let end_point = global_end_point - q_buildings.get(way.from).unwrap().translation;
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
pub enum InteractWay {
    Start {
        from: Entity,
    },
    Finish {
        from: Entity,
        connect_to: Entity,
    },
    Abort {
        aborted: Entity,
    },
}

impl InteractWay {
    pub fn handle(
        mut commands: Commands,
        mut events: EventReader<InteractWay>,
        mut controller: ResMut<WayController>,
        mut meshes: ResMut<Assets<Mesh>>,
        q_ways: Query<Entity, With<PlacingWay>>,
        q_buildings: Query<&Transform, With<Building>>,
    ) {
        for event in events.read() {
            match *event {
                InteractWay::Start {
                    from,
                } => {
                    controller.start_building = Some(from);

                    let base_mesh = meshes.get(controller.mesh.id()).unwrap().clone();
                    let mesh = meshes.add(base_mesh);

                    commands.spawn((
                        PlacingWay {
                            from,
                        },
                        PbrBundle {
                            transform: Transform::from_translation(
                                q_buildings.get(from).unwrap().translation,
                            )
                            .with_rotation(Quat::from_rotation_x(0.0)),
                            mesh,
                            material: controller.material.clone(),
                            ..default()
                        },
                    ));
                }
                InteractWay::Finish {
                    from,
                    connect_to: connected_to,
                } => {
                    let entity = q_ways.single();
                    let start_building = controller.start_building.take().unwrap();
                    controller.connected.push((start_building, connected_to));
                    commands.entity(entity).remove::<PlacingWay>();
                }
                InteractWay::Abort {
                    aborted: _,
                } => {
                    let entity = q_ways.single();
                    commands.entity(entity).despawn();
                    controller.start_building = None;
                }
            }
        }
    }
}
