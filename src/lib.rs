use bevy::{app::PluginsState, prelude::*};
use smithay_client_toolkit::{
    delegate_registry,
    globals::ProvidesBoundGlobal,
    output::OutputState,
    reexports::{
        calloop::EventLoop,
        calloop_wayland_source::WaylandSource,
        client::{globals::registry_queue_init, protocol::wl_compositor::WlCompositor, Connection},
    },
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
    seat::SeatState,
};

mod input_handler;
pub mod input_region;
pub mod layer_shell;
mod output_handler;
mod surface_handler;

#[derive(Default)]
pub struct WaylandPlugin;
impl Plugin for WaylandPlugin {
    fn build(&self, app: &mut App) {
        let connection =
            Connection::connect_to_env().expect("failed to connect to wayland socket!");
        let event_loop =
            EventLoop::<WaylandState>::try_new().expect("failed to create event_loop!");
        let (globals, event_queue) = registry_queue_init::<WaylandState>(&connection)
            .expect("failed to init registry queue");

        let qh = event_queue.handle();
        let loop_handle = event_loop.handle();
        WaylandSource::new(connection.clone(), event_queue)
            .insert(loop_handle.clone())
            .expect("failed to insert wayland source to event loop");

        app.insert_non_send_resource(RegistryState::new(&globals));
        app.insert_non_send_resource(connection.clone());
        app.insert_non_send_resource(globals);
        app.insert_non_send_resource(qh);

        app.add_plugins((
            output_handler::OutputHandlerPlugin,
            input_handler::InputHandlerPlugin,
            surface_handler::SurfaceHandlerPlugin,
            layer_shell::LayerShellPlugin,
            input_region::InputRegionPlugin,
        ));
        app.set_runner(|app| runner(app, event_loop));
    }
}

pub fn runner(mut app: App, mut event_loop: EventLoop<'_, WaylandState>) -> AppExit {
    if app.plugins_state() == PluginsState::Ready {
        app.finish();
        app.cleanup();
    }

    let mut state = WaylandState(app);
    loop {
        // TODO: Error handling
        let _ = event_loop.dispatch(None, &mut state);

        if state.plugins_state() == PluginsState::Cleaned {
            state.update();
        }
    }
}

#[derive(Deref, DerefMut)]
pub struct WaylandState(App);
impl ProvidesRegistryState for WaylandState {
    fn registry(&mut self) -> &mut smithay_client_toolkit::registry::RegistryState {
        self.world_mut()
            .non_send_resource_mut::<RegistryState>()
            .into_inner()
    }
    registry_handlers!(OutputState, SeatState);
}

delegate_registry!(WaylandState);
