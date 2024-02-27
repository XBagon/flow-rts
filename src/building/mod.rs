use bevy::app::{PluginGroup, PluginGroupBuilder};
use bevy::pbr::{ExtendedMaterial, MaterialExtension};
use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderRef};
use bevy::scene::SceneInstanceReady;

use crate::input::{InputController, InputEvent};
use crate::way::InteractWay;

use self::headquarters::HeadQuartersPlugin;
use self::tree::TreePlugin;

pub mod headquarters;
pub mod tree;

pub struct BuildingPlugins;

impl PluginGroup for BuildingPlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(BuildingPlugin)
            .add(HeadQuartersPlugin)
            .add(TreePlugin)
    }
}

pub struct BuildingPlugin;

impl Plugin for BuildingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, BuildingAssets::setup)
            .add_plugins(MaterialPlugin::<BuildingExtendedMaterial>::default())
            .add_systems(
                Update,
                (
                    BuildingAssets::on_building_scene_loaded,
                    BuildingAssets::on_instancing_scene,
                    Building::update_glowing,
                ),
            );
    }
}

#[derive(Resource)]
pub struct BuildingAssets {
    pub material: Handle<BuildingExtendedMaterial>,
}

impl BuildingAssets {
    pub fn setup(mut commands: Commands, mut materials: ResMut<Assets<BuildingExtendedMaterial>>) {
        let material = materials.add(ExtendedMaterial {
            base: StandardMaterial {
                base_color: Color::RED,
                ..Default::default()
            },
            extension: BuildingMaterialExtension {
                glowing: 0,
            },
        });
        commands.insert_resource(BuildingAssets {
            material,
        });
    }

    pub fn on_building_scene_loaded(
        mut ev_scene_asset: EventReader<AssetEvent<Scene>>,
        mut scenes: ResMut<Assets<Scene>>,
        materials: Res<Assets<StandardMaterial>>,
        mut extended_materials: ResMut<Assets<BuildingExtendedMaterial>>,
    ) {
        for event in ev_scene_asset.read() {
            if let AssetEvent::LoadedWithDependencies {
                id,
            } = *event
            {
                let scene = scenes.get_mut(id).unwrap();

                let mut material_handles =
                    scene.world.query::<(Entity, &Handle<StandardMaterial>)>();

                let to_extend = material_handles
                    .iter(&scene.world)
                    .map(|(entity, material_handle)| {
                        let material = materials.get(material_handle).unwrap();
                        let extended_material = extended_materials.add(BuildingExtendedMaterial {
                            base: material.clone(),
                            extension: BuildingMaterialExtension {
                                glowing: 0,
                            },
                        });
                        (entity, extended_material)
                    })
                    .collect::<Vec<_>>();

                for (entity, extended_material) in to_extend {
                    scene
                        .world
                        .entity_mut(entity)
                        .insert(extended_material)
                        .remove::<Handle<StandardMaterial>>();
                }
            }
        }
    }

    pub fn on_instancing_scene(
        mut ev_scene_ready: EventReader<SceneInstanceReady>,
        q_children: Query<(Entity, &Children)>,
        mut q_materials: Query<&mut Handle<BuildingExtendedMaterial>>, //or use Added filter?
        mut materials: ResMut<Assets<BuildingExtendedMaterial>>,
    ) {
        for event in ev_scene_ready.read() {
            dbg!(event);
            let (_, children) = q_children.get(event.parent).unwrap();
            //let scene_instance = q_scene_instances.get(*child).unwrap();

            //let scene = scenes.get_mut(scene_instance.).unwrap();
            let (_, children) = q_children.get(*children.first().unwrap()).unwrap();
            let (_, children) = q_children.get(*children.first().unwrap()).unwrap();

            let mut material_handles = q_materials.iter_many_mut(children);

            while let Some(mut material_handle) = material_handles.fetch_next() {
                let material = materials.get_mut(material_handle.id()).unwrap().clone();
                let new_material = materials.add(material);
                *material_handle = new_material;
                dbg!(material_handle.id());
            }
        }
    }
}

type BuildingExtendedMaterial = ExtendedMaterial<StandardMaterial, BuildingMaterialExtension>;

#[derive(Asset, AsBindGroup, Reflect, Debug, Clone)]
pub struct BuildingMaterialExtension {
    #[uniform(100)]
    glowing: u32,
}

impl MaterialExtension for BuildingMaterialExtension {
    fn fragment_shader() -> ShaderRef {
        "shaders/extended_building_shader.wgsl".into()
    }

    fn deferred_fragment_shader() -> ShaderRef {
        "shaders/extended_building_shader.wgsl".into()
    }
}

#[derive(Component)]
pub struct CustomizeMaterial {}

#[derive(Component, Debug, Default)]
pub struct Building {
    pub glowing: Glowing,
    pub connected: Vec<Entity>,
}

#[repr(u8)]
#[derive(Debug, PartialEq, Eq, Default, Copy, Clone)]
pub enum Glowing {
    #[default]
    Off = 0,
    Hovering = 1,
    Connecting = 2,
}

impl Building {
    pub fn update_glowing(
        mut ev_input: EventReader<InputEvent>,
        mut ev_interact_way: EventReader<InteractWay>,
        mut q_buildings: Query<&mut Building>,
        q_children: Query<&Children>,
        mut q_building_primitives: Query<&mut Handle<BuildingExtendedMaterial>>,
        mut materials: ResMut<Assets<BuildingExtendedMaterial>>,
        input_controller: Res<InputController>,
    ) {
        let mut modified_buildings = Vec::new();

        for event in ev_interact_way.read() {
            match *event {
                InteractWay::Start {
                    from,
                } => {
                    q_buildings.get_mut(from).unwrap().glowing = Glowing::Connecting;
                    modified_buildings.push(from);
                }
                InteractWay::Finish {
                    from,
                    connect_to,
                } => {
                    q_buildings.get_mut(from).unwrap().glowing = Glowing::Off;
                    modified_buildings.push(from);
                    q_buildings.get_mut(connect_to).unwrap().glowing = Glowing::Off;
                    modified_buildings.push(connect_to);
                }
                InteractWay::Abort {
                    aborted,
                } => {
                    q_buildings.get_mut(aborted).unwrap().glowing = Glowing::Off;
                    modified_buildings.push(aborted);
                }
            };
        }

        for event in ev_input.read() {
            if let InputEvent::ExitHoverBuilding {
                building: entity,
            } = *event
            {
                let mut building = q_buildings.get_mut(entity).unwrap();
                if building.glowing == Glowing::Hovering {
                    building.glowing = Glowing::Off;
                    modified_buildings.push(entity);
                }
            }
        }

        if let Some(entity) = input_controller.hovering_building {
            let mut building = q_buildings.get_mut(entity).unwrap();
            if building.glowing == Glowing::Off {
                building.glowing = Glowing::Hovering;
                modified_buildings.push(entity);
            }
        }

        for entity in modified_buildings {
            let building = q_buildings.get(entity).unwrap();
            let children = q_children.get(entity).unwrap();

            let children = q_children.get(*children.iter().next().unwrap()).unwrap();
            let children = q_children.get(*children.iter().next().unwrap()).unwrap();
            for child in children.iter() {
                if let Ok(material) = q_building_primitives.get_mut(*child) {
                    let material = materials.get_mut(material.id()).unwrap();

                    material.extension.glowing = building.glowing as u32;
                }
            }
        }
    }
}

#[derive(Bundle)]
pub struct BuildingBundle {
    pub building: Building,
    pub customize_material: CustomizeMaterial,
}
