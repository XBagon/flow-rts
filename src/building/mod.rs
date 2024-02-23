use bevy::app::{PluginGroup, PluginGroupBuilder};
use bevy::pbr::{ExtendedMaterial, MaterialExtension};
use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderRef};

use crate::input::{InputController, InputEvent};
use crate::way::InteractWay;

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
            .add_systems(
                Update,
                (BuildingAssets::on_building_scene_loaded, Building::update_glowing),
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
        mut scene_asset_ev: EventReader<AssetEvent<Scene>>,
        mut scenes: ResMut<Assets<Scene>>,
        materials: Res<Assets<StandardMaterial>>,
        mut extended_materials: ResMut<Assets<BuildingExtendedMaterial>>,
    ) {
        for event in scene_asset_ev.read() {
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

#[derive(Component, Debug, Default)]
pub struct Building {
    pub glowing: Glowing,
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
        q_children: Query<&Children>,
        mut q_buildings: Query<&Children>,
        mut q_building_primitives: Query<&mut Handle<BuildingExtendedMaterial>>,
        mut materials: ResMut<Assets<BuildingExtendedMaterial>>,
        input_controller: Res<InputController>,
    ) {
        let mut event_happend = false;
        for event in ev_interact_way.read() {
            event_happend = true;

            let (building, glowing) = match event {
                &InteractWay::Start {
                    from,
                } => (from, Glowing::Connecting),
                &InteractWay::Finish {
                    connect_to: building,
                }
                | &InteractWay::Abort {
                    aborted: building,
                } => (building, Glowing::Off),
            };

            let children = q_buildings.get_mut(building).unwrap();

            let children = q_children.get(*children.iter().next().unwrap()).unwrap();
            let children = q_children.get(*children.iter().next().unwrap()).unwrap();
            for child in children.iter() {
                if let Ok(material) = q_building_primitives.get_mut(*child) {
                    let material = materials.get_mut(material.id()).unwrap();
                    material.extension.glowing = glowing as u32;
                }
            }
        }

        for event in ev_input.read() {
            event_happend = true;
            if let InputEvent::ExitHoverBuilding {
                building,
            } = *event
            {
                {
                    let children = q_buildings.get_mut(building).unwrap();

                    let children = q_children.get(*children.iter().next().unwrap()).unwrap();
                    let children = q_children.get(*children.iter().next().unwrap()).unwrap();
                    for child in children.iter() {
                        if let Ok(material) = q_building_primitives.get_mut(*child) {
                            let material = materials.get_mut(material.id()).unwrap();
                            if material.extension.glowing != Glowing::Connecting as u32 {
                                material.extension.glowing = Glowing::Off as u32;
                            }
                        }
                    }
                }
            }
        }

        if event_happend {
            if let Some(hovering_building) = input_controller.hovering_building {
                let children = q_buildings.get_mut(hovering_building).unwrap();

                let children = q_children.get(*children.iter().next().unwrap()).unwrap();
                let children = q_children.get(*children.iter().next().unwrap()).unwrap();
                for child in children.iter() {
                    if let Ok(material) = q_building_primitives.get_mut(*child) {
                        let material = materials.get_mut(material.id()).unwrap();
                        if material.extension.glowing != Glowing::Connecting as u32 {
                            material.extension.glowing = Glowing::Hovering as u32;
                        }
                    }
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
