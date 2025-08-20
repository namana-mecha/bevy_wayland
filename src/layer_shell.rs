use bevy::prelude::*;
use smithay_client_toolkit::{
    delegate_layer,
    reexports::client::{globals::GlobalList, QueueHandle},
    shell::{
        wlr_layer::{Layer, LayerShell, LayerShellHandler},
        WaylandSurface,
    },
};

use crate::{surface_handler::WaylandSurfaces, WaylandState};

#[derive(Component, Default)]
pub struct LayerShellWindow {}
#[derive(Component)]
struct LayerShellRoleAssigned;

pub struct LayerShellPlugin;
impl Plugin for LayerShellPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, assign_layer_shell_role);
    }
}

fn assign_layer_shell_role(
    mut commands: Commands,
    wayland_surfaces: NonSend<WaylandSurfaces>,
    queue_handle: NonSend<QueueHandle<WaylandState>>,
    globals: NonSend<GlobalList>,
    layer_shell_windows: Query<
        (Entity, &Window, &LayerShellWindow),
        Without<LayerShellRoleAssigned>,
    >,
) {
    for (entity, _window, _layer_shell_settings) in &layer_shell_windows {
        let window_wrapper = wayland_surfaces.get_window_wrapper(entity);
        let surface = window_wrapper
            .expect("tried to assign role before creating surface!")
            .wl_surface();

        let layer_shell =
            LayerShell::bind(&globals, &queue_handle).expect("layer shell not available!");
        let layer = layer_shell.create_layer_surface(
            &queue_handle,
            surface.clone(),
            Layer::Top,
            Some("simple_layer"),
            None,
        );
        layer.commit();
        Box::leak(Box::new(layer));
        commands.entity(entity).insert(LayerShellRoleAssigned);
    }
}

impl LayerShellHandler for WaylandState {
    fn closed(
        &mut self,
        _conn: &smithay_client_toolkit::reexports::client::Connection,
        _qh: &QueueHandle<Self>,
        _layer: &smithay_client_toolkit::shell::wlr_layer::LayerSurface,
    ) {
    }

    fn configure(
        &mut self,
        _conn: &smithay_client_toolkit::reexports::client::Connection,
        _qh: &QueueHandle<Self>,
        _layer: &smithay_client_toolkit::shell::wlr_layer::LayerSurface,
        _configure: smithay_client_toolkit::shell::wlr_layer::LayerSurfaceConfigure,
        _serial: u32,
    ) {
    }
}
delegate_layer!(WaylandState);
