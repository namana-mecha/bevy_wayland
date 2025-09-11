use bevy::{platform::collections::HashMap, prelude::*};
use smithay_client_toolkit::{
    delegate_session_lock,
    output::OutputState,
    reexports::client::{globals::GlobalList, protocol::wl_output::WlOutput, QueueHandle},
    session_lock::{SessionLock, SessionLockHandler, SessionLockState, SessionLockSurface},
};

use crate::{
    surface_handler::{create_windows, SurfaceConfigured, WaylandSurfaces},
    WaylandState,
};

#[derive(Default, Deref, DerefMut)]
struct SessionLockWindows(HashMap<Entity, SessionLockWindowInternal>);
struct SessionLockWindowInternal {
    _session_lock_surface: SessionLockSurface,
}

#[derive(Component)]
pub struct SessionLockWindow;

#[derive(Component)]
struct SessionLockUnconfiguredWindow {
    output: WlOutput,
}
impl SessionLockUnconfiguredWindow {
    pub fn new(output: WlOutput) -> Self {
        Self { output }
    }
}

#[derive(Clone, Copy, Event)]
pub enum SessionLockEvent {
    Lock,
    Unlock,
}

pub struct SessionLockPlugin;
impl Plugin for SessionLockPlugin {
    fn build(&self, app: &mut App) {
        let globals = app.world().non_send_resource::<GlobalList>();
        let queue_handle = app.world().non_send_resource::<QueueHandle<WaylandState>>();
        let session_lock_state = SessionLockState::new(globals, queue_handle);

        app.insert_non_send_resource(session_lock_state);
        app.insert_non_send_resource(SessionLockWindows::default());
        app.insert_non_send_resource(SessionLockWrapper::default());
        app.add_event::<SessionLockEvent>();
        app.add_systems(
            PreUpdate,
            (
                session_lock_event_handler.before(create_windows),
                configure_lock_surfaces.after(create_windows),
            ),
        );
    }
}

#[derive(Deref, DerefMut, Default)]
struct SessionLockWrapper(Option<SessionLock>);
fn session_lock_event_handler(
    mut commands: Commands,
    mut session_lock_event_reader: EventReader<SessionLockEvent>,
    session_lock_state: NonSend<SessionLockState>,
    mut session_lock_wrapper: NonSendMut<SessionLockWrapper>,
    queue_handle: NonSend<QueueHandle<WaylandState>>,
    output_state: NonSend<OutputState>,
) {
    for session_lock_event in session_lock_event_reader.read() {
        match session_lock_event {
            SessionLockEvent::Lock => {
                if session_lock_wrapper.is_some() {
                    error!("Lock was called even if it was already aquired");
                    return;
                }
                let session_lock = session_lock_state
                    .lock(&queue_handle)
                    .expect("Unable to aquire session lock");
                let _ = session_lock_wrapper.insert(session_lock);

                for output in output_state.outputs() {
                    commands.spawn((
                        Window::default(),
                        SessionLockUnconfiguredWindow::new(output),
                    ));
                }
            }
            SessionLockEvent::Unlock => {
                if let Some(session_lock) = &**session_lock_wrapper {
                    session_lock.unlock();
                }
            }
        }
    }
}

fn configure_lock_surfaces(
    mut commands: Commands,
    mut session_lock_windows: NonSendMut<SessionLockWindows>,
    session_lock_wrapper: NonSend<SessionLockWrapper>,
    wayland_surfaces: NonSend<WaylandSurfaces>,
    qh: NonSend<QueueHandle<WaylandState>>,
    unconfigured_windows: Query<(Entity, &SessionLockUnconfiguredWindow)>,
) {
    if let Some(session_lock) = &**session_lock_wrapper {
        for (entity, unconfigured_window) in &unconfigured_windows {
            let window_wrapper = wayland_surfaces.get_window_wrapper(entity);
            let surface = window_wrapper
                .expect("tried to assign role before creating surface!")
                .wl_surface();
            let _session_lock_surface =
                session_lock.create_lock_surface(surface.clone(), &unconfigured_window.output, &qh);

            let session_lock_window = SessionLockWindowInternal {
                _session_lock_surface,
            };

            session_lock_windows.insert(entity, session_lock_window);
            commands
                .entity(entity)
                .insert(SurfaceConfigured)
                .insert(SessionLockWindow)
                .remove::<SessionLockUnconfiguredWindow>();
        }
    }
}

impl SessionLockHandler for WaylandState {
    fn locked(
        &mut self,
        _conn: &smithay_client_toolkit::reexports::client::Connection,
        _qh: &smithay_client_toolkit::reexports::client::QueueHandle<Self>,
        _session_lock: smithay_client_toolkit::session_lock::SessionLock,
    ) {
    }

    fn finished(
        &mut self,
        _conn: &smithay_client_toolkit::reexports::client::Connection,
        _qh: &smithay_client_toolkit::reexports::client::QueueHandle<Self>,
        _session_lock: smithay_client_toolkit::session_lock::SessionLock,
    ) {
    }

    fn configure(
        &mut self,
        _conn: &smithay_client_toolkit::reexports::client::Connection,
        _qh: &smithay_client_toolkit::reexports::client::QueueHandle<Self>,
        _surface: smithay_client_toolkit::session_lock::SessionLockSurface,
        _configure: smithay_client_toolkit::session_lock::SessionLockSurfaceConfigure,
        _serial: u32,
    ) {
    }
}
delegate_session_lock!(WaylandState);
