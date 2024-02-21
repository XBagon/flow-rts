use bevy::app::{PluginGroup, PluginGroupBuilder};
use bevy::pbr::{ExtendedMaterial, MaterialExtension};
use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderRef};

use crate::input::HoveringBuildingChanged;

use self::headquarters::HeadQuartersPlugin;

pub mod headquarters;

pub struct BuildingPlugins;

impl PluginGroup for BuildingPlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>().add(BuildingPlugin).add(HeadQuartersPlugin)
    }
}

pub struct BuildingPlugin;

impl Plugin for BuildingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, BuildingAssets::setup)
            .add_plugins(MaterialPlugin::<BuildingExtendedMaterial>::default())
            .add_systems(Update, (BuildingAssets::on_building_scene_loaded, Building::update));
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
        mut scene_asset_ev: EventReader<AssetEvent<Scene>>,
        mut scenes: ResMut<Assets<Scene>>,
        materials: Res<Assets<StandardMaterial>>,
        mut extended_materials: ResMut<Assets<BuildingExtendedMaterial>>,
    ) {
        for event in scene_asset_ev.read() {
            match event {
                AssetEvent::LoadedWithDependencies {
                    id,
                } => {
                    let scene = scenes.get_mut(*id).unwrap();

                    let mut material_handles =
                        scene.world.query::<(Entity, &Handle<StandardMaterial>)>();

                    let to_extend = material_handles
                        .iter(&scene.world)
                        .map(|(entity, material_handle)| {
                            let material = materials.get(material_handle).unwrap();
                            let extended_material =
                                extended_materials.add(BuildingExtendedMaterial {
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
                _ => {}
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
        "shaders\\extended_building_shader.wgsl".into()
    }

    fn deferred_fragment_shader() -> ShaderRef {
        "shaders\\extended_building_shader.wgsl".into()
    }
}

#[derive(Component)]
pub struct CustomizeMaterial {}

#[derive(Component, Debug)]
pub struct Building {}

impl Building {
    pub fn update(
        mut start_hovering: EventReader<HoveringBuildingChanged>,
        q_children: Query<&Children>,
        mut q_buildings: Query<(&Building, &Children)>,
        mut q_building_primitives: Query<&mut Handle<BuildingExtendedMaterial>>,
        mut materials: ResMut<Assets<BuildingExtendedMaterial>>,
    ) {
        if let Some(start_hovering) = start_hovering.read().next() {
            let (_building, children) = q_buildings.get_mut(start_hovering.building).unwrap();
            let children = q_children.get(*children.iter().next().unwrap()).unwrap();
            let children = q_children.get(*children.iter().next().unwrap()).unwrap();
            for child in children.iter() {
                if let Ok(material) = q_building_primitives.get_mut(*child) {
                    let material = materials.get_mut(material.id()).unwrap();
                    material.extension.glowing = start_hovering.active as u32;
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
