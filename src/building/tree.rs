use bevy::transform::TransformSystem::TransformPropagate;
use bevy::{math::Affine3A, prelude::*};
use bevy_xpbd_3d::plugins::collision::Collider;

use super::Building;
use crate::unit::{SpawnUnit, Unit, UnitArrived};

pub struct TreePlugin;

impl Plugin for TreePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, TreeSpawner::setup)
            .add_event::<SpawnTree>()
            .add_systems(Update, (SpawnTree::handle, Tree::update))
            .add_systems(PostUpdate, Tree::unit_arrived.after(TransformPropagate));
    }
}

#[derive(Resource, Reflect)]
pub struct TreeSpawner {
    pub scene: Handle<Scene>,
}

impl TreeSpawner {
    pub fn setup(mut commands: Commands, asset_server: ResMut<AssetServer>) {
        let scene = asset_server.load("models/detail_treeLarge.glb#Scene0");

        commands.insert_resource(TreeSpawner {
            scene,
        });
    }
}

#[derive(Event)]
pub struct SpawnTree {
    pub position: Vec3,
}

impl SpawnTree {
    pub fn handle(
        mut commands: Commands,
        head_quaters_spawner: ResMut<TreeSpawner>,
        mut spawn_head_quarters: EventReader<SpawnTree>,
    ) {
        for event in spawn_head_quarters.read() {
            commands.spawn((
                Tree {},
                Building::default(),
                SceneBundle {
                    scene: head_quaters_spawner.scene.clone(),
                    transform: Transform::from_translation(event.position)
                        .with_scale(Vec3::splat(2.0)),
                    ..default()
                },
                Collider::cuboid(1.0, 1.0, 1.0),
            ));
        }
    }
}

#[derive(Component)]
pub struct Tree {}

impl Tree {
    pub fn update(time: Res<Time>, mut trees: Query<(Entity, &mut Tree, &Building, &Transform)>) {}

    pub fn unit_arrived(
        mut commands: Commands,
        mut ev_unit_arrived: EventReader<UnitArrived>,
        mut trees: Query<(&mut Tree, &mut GlobalTransform)>,
    ) {
        //for event in ev_unit_arrived.read() {
        //let (tree, mut global_transform) = trees.get_mut(event.building).unwrap();
        for (tree, mut global_transform) in trees.iter_mut() {
            let shearing_y = 10.5;
            let shearing_z = 10.5;
            let shear_affine = Affine3A::from_cols_array_2d(&[
                [1.0, shearing_y, shearing_z],
                [0.0, 1.0, 0.0],
                [0.0, 0.0, 1.0],
                [0.0, 0.0, 0.0],
            ]);
            *global_transform = GlobalTransform::from(global_transform.affine() * shear_affine);
        }
    }
}
