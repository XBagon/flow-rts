use bevy::prelude::*;

use crate::{
    building::Building,
    way::{FinishWay, StartWay, WayController},
};

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InputController>()
            .add_event::<HoveringBuildingChanged>()
            .add_systems(Update, update);
    }
}

#[derive(Default, Resource)]
pub struct InputController {
    pub plane_position: Option<Vec3>,
    pub hovering_building: Option<Entity>,
}

pub fn update(
    mut controller: ResMut<InputController>,
    buttons: Res<ButtonInput<MouseButton>>,
    way_controller: Res<WayController>,
    mut start_way_evs: EventWriter<StartWay>,
    mut finish_way_evs: EventWriter<FinishWay>,
    q_camera: Query<(&Camera, &GlobalTransform)>,
    q_window: Query<&Window>,
    q_buildings: Query<(Entity, &Transform), With<Building>>,
    mut hovering_building_ev: EventWriter<HoveringBuildingChanged>,
) {
    if buttons.just_pressed(MouseButton::Left) {
        if way_controller.currenty_placing {
            finish_way_evs.send(FinishWay {
                abort: false,
            });
        } else {
            if let Some(plane_position) = controller.plane_position {
                start_way_evs.send(StartWay {
                    from: plane_position,
                });
            }
        }
    }
    if buttons.just_pressed(MouseButton::Right) {
        finish_way_evs.send(FinishWay {
            abort: true,
        });
    }

    let (camera, camera_transform) = q_camera.single();

    // There is only one primary window, so we can similarly get it from the query:
    let window = q_window.single();

    // check if the cursor is inside the window and get its position
    let Some(cursor_position) = window.cursor_position() else {
        // if the cursor is not inside the window, we can't do anything
        return;
    };

    // Ask Bevy to give us a ray pointing from the viewport (screen) into the world
    let Some(ray) = camera.viewport_to_world(camera_transform, cursor_position) else {
        // if it was impossible to compute for whatever reason; we can't do anything
        return;
    };

    let ground = Plane3d::default();

    // do a ray-plane intersection test, giving us the distance to the ground
    let Some(distance) = ray.intersect_plane(Vec3::ZERO, ground) else {
        // If the ray does not intersect the ground
        // (the camera is not looking towards the ground), we can't do anything
        return;
    };

    // use the distance to compute the actual point on the ground in world-space
    let global_cursor = ray.get_point(distance);

    controller.plane_position = Some(global_cursor);

    // to compute the local coordinates, we need the inverse of the plane's transform
    //let inverse_transform_matrix = ground_transform.compute_matrix().inverse();
    //let local_cursor = inverse_transform_matrix.transform_point3(global_cursor);

    // we can discard the Y coordinate, because it should always be zero
    // (our point is supposed to be on the plane)
    //mycoords.local = local_cursor.xz();
    //eprintln!("Local cursor coords: {}/{}", local_cursor.x, local_cursor.z);

    for (entity, transform) in q_buildings.iter() {
        let distance_vector = global_cursor - transform.translation;
        if distance_vector.x.abs() < 0.5 && distance_vector.z.abs() < 0.5 {
            if Some(entity) != controller.hovering_building {
                hovering_building_ev.send(HoveringBuildingChanged {
                    building: entity,
                    active: true,
                });
                controller.hovering_building = Some(entity);
            }
            return;
        }
    }

    if let Some(hovering_building) = controller.hovering_building {
        hovering_building_ev.send(HoveringBuildingChanged {
            building: hovering_building,
            active: false,
        });
        controller.hovering_building = None;
    }
}

#[derive(Event)]
pub struct HoveringBuildingChanged {
    pub building: Entity,
    pub active: bool,
}
