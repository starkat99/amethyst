pub use crate::{
    camera::ActiveCamera, legion::camera::ActiveCamera as LegionActiveCamera, types::Backend,
};
use amethyst_assets::{AssetStorage, Handle};
use amethyst_core::{
    ecs::{self as specs, SystemData, WorldExt},
    legion::{dispatcher::DispatcherBuilder, sync::SyncDirection, LegionState, LegionSyncBuilder},
};

use derivative::Derivative;
use rendy::factory::Factory;
use std::marker::PhantomData;

pub mod bundle;
pub mod pass;
pub mod plugins;
pub mod sprite_visibility;
pub mod submodules;
pub mod system;
pub mod visibility;

pub mod camera {
    #[derive(Clone, Debug, PartialEq, Default)]
    pub struct ActiveCamera {
        /// Camera entity
        pub entity: Option<amethyst_core::legion::Entity>,
    }
}

#[derive(Derivative)]
#[derivative(Default(bound = ""))]
pub struct Syncer<B: Backend>(PhantomData<B>);
impl<B: Backend> LegionSyncBuilder for Syncer<B> {
    fn prepare(
        &mut self,
        specs_world: &mut specs::World,
        world: &mut LegionState,
        dispatcher: &mut DispatcherBuilder<'_>,
    ) {
        crate::system::SetupData::setup(specs_world);

        specs_world.register::<ActiveCamera>();
        world.resources.insert(camera::ActiveCamera::default());

        world.add_component_sync_with(
            |direction,
             bimap,
             specs: Option<&mut ActiveCamera>,
             legion: Option<&mut LegionActiveCamera>| {
                let bimap = bimap.read().unwrap();

                match direction {
                    SyncDirection::SpecsToLegion => {
                        let specs_camera = specs.unwrap();
                        let legion_entity = specs_camera
                            .entity
                            .map(|s| *bimap.get_by_right(&s).unwrap());

                        if let Some(legion_camera) = legion {
                            legion_camera.entity = legion_entity;
                            return (None, None);
                        } else {
                            return (
                                None,
                                Some(LegionActiveCamera {
                                    entity: legion_entity,
                                }),
                            );
                        }
                    }
                    SyncDirection::LegionToSpecs => {
                        let legion_camera = legion.unwrap();
                        let specs_entity = legion_camera
                            .entity
                            .map(|s| *bimap.get_by_left(&s).unwrap());

                        if let Some(specs_camera) = specs {
                            specs_camera.entity = specs_entity;
                            return (None, None);
                        } else {
                            return (
                                Some(ActiveCamera {
                                    entity: specs_entity,
                                }),
                                None,
                            );
                        }
                    }
                }
            },
        );

        world.add_component_sync::<crate::SpriteRender>();
        world.add_component_sync::<crate::visibility::BoundingSphere>();
        world.add_component_sync::<crate::Camera>();
        world.add_component_sync::<crate::Transparent>();
        world.add_component_sync::<crate::resources::Tint>();
        world.add_component_sync::<crate::light::Light>(); // TODO: This causes chunk index out of bounds, why?
        world.add_component_sync::<crate::debug_drawing::DebugLinesComponent>();
        world.add_component_sync::<crate::skinning::JointTransforms>();
        world.add_component_sync::<Handle<crate::mtl::Material>>();
        world.add_component_sync::<Handle<crate::Mesh>>();
        world.add_component_sync::<crate::visibility::BoundingSphere>();

        world.add_component_sync::<Handle<crate::mtl::Material>>();
        world.add_component_sync::<Handle<crate::Mesh>>();

        world.add_resource_sync::<AssetStorage<crate::mtl::Material>>();
        world.add_resource_sync::<AssetStorage<crate::Mesh>>();
        world.add_resource_sync::<AssetStorage<crate::Texture>>();
        world.add_resource_sync::<AssetStorage<crate::sprite::SpriteSheet>>();

        world.add_resource_sync::<amethyst_assets::HotReloadStrategy>();
        world.add_resource_sync::<rendy::command::QueueId>();

        world.add_resource_sync::<crate::visibility::Visibility>();
        world.add_resource_sync::<crate::MaterialDefaults>();

        world.add_resource_sync::<amethyst_assets::Loader>();

        world.add_resource_sync::<Factory<B>>();

        world.add_resource_sync::<crate::debug_drawing::DebugLines>();
    }
}
