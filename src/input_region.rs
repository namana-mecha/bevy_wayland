use bevy::{core_pipeline::core_2d::graph::input, prelude::*};
use smithay_client_toolkit::{
    compositor::{CompositorState, Region},
    reexports::client::{protocol::wl_compositor::WlCompositor, QueueHandle},
};

use crate::{input_region, surface_handler::WaylandSurfaces, WaylandState};

#[derive(Component, Deref)]
pub struct InputRegion(pub Rect);

pub struct InputRegionPlugin;
impl Plugin for InputRegionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_input_region);
    }
}

fn update_input_region(
    windows: Query<(Entity, Option<&InputRegion>), With<Window>>,
    compositor: NonSendMut<CompositorState>,
    wayland_surfaces: NonSendMut<WaylandSurfaces>,
) {
    for (entity, input_region) in &windows {
        let window_wrapper = wayland_surfaces.get_window_wrapper(entity).unwrap();
        let region = input_region.map(|input_region| {
            let region = Region::new(compositor.as_ref()).unwrap();
            region.add(
                input_region.min.x as i32,
                input_region.min.y as i32,
                input_region.width() as i32,
                input_region.height() as i32,
            );
            region
        });
        if let Some(region) = region {
            window_wrapper
                .wl_surface()
                .set_input_region(Some(region.wl_region()));
        } else {
            window_wrapper.wl_surface().set_input_region(None);
        }
    }
}
