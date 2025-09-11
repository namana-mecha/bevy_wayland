use bevy::prelude::*;
use smithay_client_toolkit::{
    reexports::client::{event_created_child, Dispatch, QueueHandle},
    registry::RegistryState,
};
use wayland_protocols_wlr::foreign_toplevel::v1::client::{
    zwlr_foreign_toplevel_handle_v1::{self, ZwlrForeignToplevelHandleV1},
    zwlr_foreign_toplevel_manager_v1::{self, ZwlrForeignToplevelManagerV1},
};

use crate::WaylandState;
#[derive(Debug, Copy, Clone, Event)]
pub enum ForeignToplevelEvent {
    MinimizeOthers,
}

#[derive(Default, Deref, DerefMut)]
struct ForeignToplevels(Vec<ZwlrForeignToplevelHandleV1>);

pub struct ForeignToplevelManagerPlugin;
impl Plugin for ForeignToplevelManagerPlugin {
    fn build(&self, app: &mut App) {
        let registry_state = app.world().non_send_resource::<RegistryState>();
        let queue_handle = app.world().non_send_resource::<QueueHandle<WaylandState>>();
        let foreign_top_level_manager =
            registry_state.bind_one::<ZwlrForeignToplevelManagerV1, _, _>(queue_handle, 2..=3, ());
        if let Ok(foreign_top_level_manager) = foreign_top_level_manager {
            info!("Foreign toplevel manager was bound!");
            app.insert_non_send_resource(foreign_top_level_manager);
            app.insert_non_send_resource(ForeignToplevels::default());
            app.add_event::<ForeignToplevelEvent>();
            app.add_systems(Update, foreign_top_level_event_handler);
        } else {
            let bind_error = foreign_top_level_manager.err().unwrap();
            error!("Couldn't bind foreign toplevel manager! {:?}", bind_error);
        }
    }
}

fn foreign_top_level_event_handler(
    foreign_top_levels: NonSendMut<ForeignToplevels>,
    mut events: EventReader<ForeignToplevelEvent>,
) {
    for event in events.read() {
        match event {
            ForeignToplevelEvent::MinimizeOthers => {
                info!("Minimizing other windows");
                for toplevel in foreign_top_levels.iter() {
                    toplevel.set_minimized();
                }
            }
        }
    }
}

impl Dispatch<ZwlrForeignToplevelManagerV1, ()> for WaylandState {
    fn event(
        state: &mut Self,
        _proxy: &ZwlrForeignToplevelManagerV1,
        event: <ZwlrForeignToplevelManagerV1 as smithay_client_toolkit::reexports::client::Proxy>::Event,
        _data: &(),
        _conn: &smithay_client_toolkit::reexports::client::Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
        let mut foreign_toplevels = state
            .world_mut()
            .non_send_resource_mut::<ForeignToplevels>();
        match event {
            wayland_protocols_wlr::foreign_toplevel::v1::client::zwlr_foreign_toplevel_manager_v1::Event::Toplevel { toplevel } => {
                foreign_toplevels.push(toplevel);
            },
            wayland_protocols_wlr::foreign_toplevel::v1::client::zwlr_foreign_toplevel_manager_v1::Event::Finished => {},
            _ => {},
        }
    }

    event_created_child!(WaylandState, ZwlrForeignToplevelManagerV1, [
        // Opcode 0 is the `toplevel` event. It creates a new `zwlr_foreign_toplevel_handle_v1`.
        zwlr_foreign_toplevel_manager_v1::EVT_TOPLEVEL_OPCODE => (
            zwlr_foreign_toplevel_handle_v1::ZwlrForeignToplevelHandleV1,
            ()
        )
    ]);
}

impl Dispatch<ZwlrForeignToplevelHandleV1, ()> for WaylandState {
    fn event(
        _state: &mut Self,
        _proxy: &ZwlrForeignToplevelHandleV1,
        _event: <ZwlrForeignToplevelHandleV1 as smithay_client_toolkit::reexports::client::Proxy>::Event,
        _data: &(),
        _conn: &smithay_client_toolkit::reexports::client::Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
    }
}
