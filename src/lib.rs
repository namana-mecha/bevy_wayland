use std::{
    num::NonZero,
    sync::mpsc::SendError,
    time::{Duration, Instant},
};

use bevy::{app::PluginsState, prelude::*};
use smithay_client_toolkit::{
    delegate_registry,
    output::OutputState,
    reexports::{
        calloop::{self, channel::Sender, EventLoop},
        calloop_wayland_source::WaylandSource,
        client::{globals::registry_queue_init, Connection},
    },
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
    seat::SeatState,
};

mod input_handler;
pub mod input_region;
pub mod layer_shell;
mod output_handler;
pub mod session_lock;
mod surface_handler;

pub mod prelude {
    pub use crate::input_region::InputRegion;
    pub use crate::layer_shell::{LayerShellSettings, LayerShellWindowSize};
    pub use crate::session_lock::{SessionLockEvent, SessionLockWindow};
    pub use crate::WaylandPlugin;
    pub use smithay_client_toolkit::shell::wlr_layer::{Anchor, KeyboardInteractivity, Layer};
}

pub struct Tick;
#[derive(Resource, Clone)]
pub struct ExternalEventDispatcher(Sender<Tick>);
impl ExternalEventDispatcher {
    fn new(tx: Sender<Tick>) -> Self {
        Self(tx)
    }

    pub fn dispatch(&self) -> Result<(), SendError<Tick>> {
        self.0.send(Tick)
    }
}
#[derive(Default)]
pub struct WaylandPlugin;
impl Plugin for WaylandPlugin {
    fn build(&self, app: &mut App) {
        let connection =
            Connection::connect_to_env().expect("Failed to connect to wayland socket!");
        let event_loop =
            EventLoop::<WaylandState>::try_new().expect("Failed to create event_loop!");
        let (globals, event_queue) = registry_queue_init::<WaylandState>(&connection)
            .expect("Failed to init registry queue");

        let qh = event_queue.handle();
        let loop_handle = event_loop.handle();
        WaylandSource::new(connection.clone(), event_queue)
            .insert(loop_handle.clone())
            .expect("Failed to insert wayland source to event loop");

        let (tx, rx) = calloop::channel::channel::<Tick>();
        loop_handle
            .insert_source(rx, |_, _, state| {
                info!("External event was received!");
                if state.plugins_state() == PluginsState::Cleaned {
                    state.update();
                }
            })
            .expect("Failed to insert external tick channel!");

        app.insert_resource(ExternalEventDispatcher::new(tx));
        app.insert_non_send_resource(RegistryState::new(&globals));
        app.insert_non_send_resource(connection.clone());
        app.insert_non_send_resource(globals);
        app.insert_non_send_resource(qh);

        app.add_plugins((
            output_handler::OutputHandlerPlugin,
            surface_handler::SurfaceHandlerPlugin,
            input_handler::InputHandlerPlugin,
            layer_shell::LayerShellPlugin,
            session_lock::SessionLockPlugin,
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
        let frame_start = Instant::now();
        let _ = event_loop.dispatch(Duration::from_millis(5000), &mut state);
        if state.plugins_state() == PluginsState::Cleaned {
            state.update();
        }
        let _ = event_loop.dispatch(Duration::from_millis(0), &mut state);
        // TODO: Poll until delta time is greater than target frame time.
        if Instant::now() - frame_start < Duration::from_millis(16) {
            std::thread::sleep(Duration::from_millis(16) - (frame_start - Instant::now()));
        }
        let _ = event_loop.dispatch(Duration::from_millis(0), &mut state);
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
