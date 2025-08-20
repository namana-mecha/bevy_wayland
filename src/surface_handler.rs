use std::collections::HashMap;

use bevy::{
    ecs::entity::EntityHashMap,
    prelude::*,
    window::{RawHandleWrapper, RawHandleWrapperHolder, WindowCreated, WindowWrapper},
};
use raw_window_handle::{
    DisplayHandle, HasDisplayHandle, HasWindowHandle, RawDisplayHandle, RawWindowHandle,
    WaylandDisplayHandle, WaylandWindowHandle, WindowHandle,
};
use smithay_client_toolkit::{
    compositor::{CompositorHandler, CompositorState},
    delegate_compositor,
    reexports::client::{
        backend::ObjectId, protocol::wl_surface::WlSurface, Connection, Proxy, QueueHandle,
    },
};

use crate::WaylandState;

pub struct SurfaceHandlerPlugin;
impl Plugin for SurfaceHandlerPlugin {
    fn build(&self, app: &mut App) {
        let queue_handle: &QueueHandle<WaylandState> = app.world().non_send_resource();
        let globals = app.world().non_send_resource();
        app.insert_non_send_resource(
            CompositorState::bind(globals, queue_handle).expect("failed to bind compositor!"),
        );
        app.insert_non_send_resource(WaylandSurfaces::default());
        app.add_systems(PreUpdate, create_windows);
    }
}

impl CompositorHandler for WaylandState {
    fn scale_factor_changed(
        &mut self,
        _conn: &smithay_client_toolkit::reexports::client::Connection,
        _qh: &QueueHandle<Self>,
        _surface: &smithay_client_toolkit::reexports::client::protocol::wl_surface::WlSurface,
        _new_factor: i32,
    ) {
    }

    fn transform_changed(
        &mut self,
        _conn: &smithay_client_toolkit::reexports::client::Connection,
        _qh: &QueueHandle<Self>,
        _surface: &smithay_client_toolkit::reexports::client::protocol::wl_surface::WlSurface,
        _new_transform: smithay_client_toolkit::reexports::client::protocol::wl_output::Transform,
    ) {
    }

    fn frame(
        &mut self,
        _conn: &smithay_client_toolkit::reexports::client::Connection,
        _qh: &QueueHandle<Self>,
        _surface: &smithay_client_toolkit::reexports::client::protocol::wl_surface::WlSurface,
        _time: u32,
    ) {
    }

    fn surface_enter(
        &mut self,
        _conn: &smithay_client_toolkit::reexports::client::Connection,
        _qh: &QueueHandle<Self>,
        _surface: &smithay_client_toolkit::reexports::client::protocol::wl_surface::WlSurface,
        _output: &smithay_client_toolkit::reexports::client::protocol::wl_output::WlOutput,
    ) {
    }

    fn surface_leave(
        &mut self,
        _conn: &smithay_client_toolkit::reexports::client::Connection,
        _qh: &QueueHandle<Self>,
        _surface: &smithay_client_toolkit::reexports::client::protocol::wl_surface::WlSurface,
        _output: &smithay_client_toolkit::reexports::client::protocol::wl_output::WlOutput,
    ) {
    }
}
delegate_compositor!(WaylandState);

#[derive(Default)]
pub struct WaylandSurfaces {
    windows: HashMap<ObjectId, WindowWrapper<WaylandSurface>>,
    entity_to_surface: EntityHashMap<ObjectId>,
    surface_to_entity: HashMap<ObjectId, Entity>,

    _not_send_sync: core::marker::PhantomData<*const ()>,
}

impl WaylandSurfaces {
    pub fn create_surface(
        &mut self,
        entity: Entity,
        queue_handle: &QueueHandle<WaylandState>,
        connection: Connection,
        compositor_state: &CompositorState,
    ) -> &WindowWrapper<WaylandSurface> {
        let wl_surface = compositor_state.create_surface(queue_handle);
        let wayland_surface = WaylandSurface::new(wl_surface, connection);
        let surface_id = wayland_surface.id();
        self.windows
            .insert(wayland_surface.id(), WindowWrapper::new(wayland_surface));

        self.entity_to_surface.insert(entity, surface_id.clone());
        self.surface_to_entity.insert(surface_id.clone(), entity);
        self.windows.get(&surface_id).unwrap()
    }

    pub fn get_window_wrapper(&self, entity: Entity) -> Option<&WindowWrapper<WaylandSurface>> {
        self.entity_to_surface
            .get(&entity)
            .map(|surface_id| self.windows.get(surface_id))?
    }
}

pub struct WaylandSurface {
    surface: WlSurface,
    connection: Connection,
}

impl WaylandSurface {
    pub fn new(surface: WlSurface, connection: Connection) -> Self {
        Self {
            surface,
            connection,
        }
    }

    pub fn wl_surface(&self) -> &WlSurface {
        &self.surface
    }

    pub fn id(&self) -> ObjectId {
        self.surface.id()
    }
}

impl HasWindowHandle for WaylandSurface {
    fn window_handle(
        &self,
    ) -> std::result::Result<raw_window_handle::WindowHandle<'_>, raw_window_handle::HandleError>
    {
        let raw_window_handle = RawWindowHandle::Wayland(WaylandWindowHandle::new(
            core::ptr::NonNull::new(self.wl_surface().id().as_ptr() as *mut _).unwrap(),
        ));
        unsafe { Ok(WindowHandle::borrow_raw(raw_window_handle)) }
    }
}

impl HasDisplayHandle for WaylandSurface {
    fn display_handle(
        &self,
    ) -> Result<raw_window_handle::DisplayHandle<'_>, raw_window_handle::HandleError> {
        let raw_display_handle = RawDisplayHandle::Wayland(WaylandDisplayHandle::new(
            core::ptr::NonNull::new(self.connection.backend().display_ptr() as *mut _).unwrap(),
        ));
        unsafe { Ok(DisplayHandle::borrow_raw(raw_display_handle)) }
    }
}

pub fn create_windows(
    mut commands: Commands,
    mut wayland_surfaces: NonSendMut<WaylandSurfaces>,
    compositor_state: NonSend<CompositorState>,
    connection: NonSend<Connection>,
    queue_handle: NonSend<QueueHandle<WaylandState>>,
    bevy_windows: Query<(Entity, Option<&RawHandleWrapperHolder>), With<Window>>,
    mut window_created_event: EventWriter<WindowCreated>,
) {
    for (entity, handle_holder) in &bevy_windows {
        if wayland_surfaces.get_window_wrapper(entity).is_some() {
            continue;
        }
        println!("Creating Window");

        let surface = wayland_surfaces.create_surface(
            entity,
            &queue_handle,
            connection.clone(),
            &compositor_state,
        );
        let mut wrapper: Option<_> = None;
        if let Ok(handle_wrapper) = RawHandleWrapper::new(surface) {
            wrapper = Some(handle_wrapper.clone());
            if let Some(handle_holder) = handle_holder {
                *handle_holder.0.lock().unwrap() = Some(handle_wrapper);
            }
        }
        commands.entity(entity).insert(wrapper.unwrap());
        window_created_event.write(WindowCreated { window: entity });
    }
}
