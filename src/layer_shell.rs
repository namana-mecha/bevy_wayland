use bevy::{platform::collections::HashMap, prelude::*, ui::update};
use smithay_client_toolkit::{
    delegate_layer,
    reexports::client::{globals::GlobalList, Connection, Proxy, QueueHandle},
    shell::{
        wlr_layer::{
            Anchor, KeyboardInteractivity, Layer, LayerShell, LayerShellHandler, LayerSurface,
        },
        WaylandSurface,
    },
};

use crate::{
    surface_handler::{create_windows, SurfaceConfigured, WaylandSurfaces},
    WaylandState,
};

#[derive(Default, Deref, DerefMut)]
struct LayerShellWindows(HashMap<Entity, LayerShellWindow>);

struct LayerShellWindow {
    layer_surface: LayerSurface,
    layer_shell_settings: LayerShellSettings,
}
impl LayerShellWindow {
    fn new(layer_surface: LayerSurface, layer_shell_settings: LayerShellSettings) -> Self {
        let mut layer_shell_window = Self {
            layer_surface,
            layer_shell_settings,
        };
        layer_shell_window.sync();
        layer_shell_window
    }

    fn sync(&mut self) {
        self.layer_surface
            .set_layer(self.layer_shell_settings.layer);
        self.layer_surface
            .set_anchor(self.layer_shell_settings.anchor);
        self.layer_surface
            .set_keyboard_interactivity(self.layer_shell_settings.keyboard_interactivity);
        self.layer_surface
            .set_exclusive_zone(self.layer_shell_settings.exclusive_zone);

        let (width, height) = self.layer_shell_settings.size;
        self.layer_surface.set_size(400, 400);

        let (top, right, bottom, left) = self.layer_shell_settings.margin;
        self.layer_surface.set_margin(top, right, bottom, left);
        self.layer_surface.commit();
    }

    pub fn set_settings(&mut self, layer_shell_settings: LayerShellSettings) {
        if self.layer_shell_settings == layer_shell_settings {
            return;
        }
        self.layer_shell_settings = layer_shell_settings;
        self.sync();
    }
}

#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub struct LayerShellSettings {
    /// Defines where the layer surface should be anchored to the screen.
    ///
    /// You can anchor the layer surface to any combination of the top, bottom, left, and right edges of the screen.
    pub anchor: Anchor,
    /// Defines the size of the layer surface in pixels.
    pub size: (u32, u32),
    /// Defines the amount of exclusive space the layer surface should reserve.
    ///
    /// Other surfaces will not be placed in this area. A negative value means that the layer surface
    /// will not reserve any exclusive space.
    pub exclusive_zone: i32,
    /// Defines the margins for the layer surface.
    ///
    /// Margins are specified in the order: top, right, bottom, left.
    pub margin: (i32, i32, i32, i32),
    /// Defines how the layer surface should handle keyboard interactivity.
    ///
    /// If set to `Exclusive`, the layer surface will receive all keyboard input.
    /// If set to `OnDemand`, the layer surface will only receive keyboard input when it is focused.
    /// If set to `None`, the layer surface will never receive keyboard input.
    pub keyboard_interactivity: KeyboardInteractivity,
    /// Defines the layer that the surface should be placed on.
    ///
    /// The layer determines the stacking order of the surface. Surfaces on higher layers are
    /// always drawn on top of surfaces on lower layers.
    pub layer: Layer,
}
impl Default for LayerShellSettings {
    fn default() -> Self {
        Self {
            anchor: Anchor::empty(),
            size: Default::default(),
            exclusive_zone: Default::default(),
            margin: Default::default(),
            keyboard_interactivity: KeyboardInteractivity::OnDemand,
            layer: Layer::Top,
        }
    }
}

pub struct LayerShellPlugin;
impl Plugin for LayerShellPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, assign_layer_shell_role.after(create_windows))
            .add_systems(Update, update_layer_shell_settings)
            .insert_non_send_resource(LayerShellWindows::default());
    }
}

fn assign_layer_shell_role(
    mut commands: Commands,
    wayland_surfaces: NonSend<WaylandSurfaces>,
    queue_handle: NonSend<QueueHandle<WaylandState>>,
    globals: NonSend<GlobalList>,
    windows: Query<(Entity, &Window, &LayerShellSettings), Without<SurfaceConfigured>>,
    mut layer_shell_windows: NonSendMut<LayerShellWindows>,
) {
    for (entity, _window, layer_shell_settings) in &windows {
        let window_wrapper = wayland_surfaces.get_window_wrapper(entity);
        let surface = window_wrapper
            .expect("tried to assign role before creating surface!")
            .wl_surface();

        let layer_shell =
            LayerShell::bind(&globals, &queue_handle).expect("layer shell not available!");
        let layer = layer_shell.create_layer_surface(
            &queue_handle,
            surface.clone(),
            layer_shell_settings.layer,
            Some("simple_layer"),
            None,
        );

        let _ = layer_shell_windows.insert(
            entity,
            LayerShellWindow::new(layer, layer_shell_settings.clone()),
        );

        commands.entity(entity).insert(SurfaceConfigured);
    }
}

fn update_layer_shell_settings(
    mut layer_shell_windows: NonSendMut<LayerShellWindows>,
    windows: Query<(Entity, &Window, &LayerShellSettings), Without<SurfaceConfigured>>,
) {
    for (entity, _window, layer_shell_settings) in &windows {
        let layer_shell_window = layer_shell_windows.get_mut(&entity).unwrap();
        layer_shell_window.set_settings(layer_shell_settings.clone());
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
