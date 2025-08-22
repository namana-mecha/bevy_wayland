use bevy::prelude::*;
use smithay_client_toolkit::{
    delegate_seat,
    reexports::client::{
        protocol::{wl_keyboard::WlKeyboard, wl_pointer::WlPointer},
        QueueHandle,
    },
    seat::{Capability, SeatHandler, SeatState},
};

use crate::WaylandState;

mod keyboard;
mod pointer;

pub struct InputHandlerPlugin;
impl Plugin for InputHandlerPlugin {
    fn build(&self, app: &mut App) {
        let globals = app.world().non_send_resource();
        let queue_handle: &QueueHandle<WaylandState> = app.world().non_send_resource();
        let seat_state = SeatState::new(globals, queue_handle);

        app.insert_non_send_resource(seat_state);
    }
}

impl SeatHandler for WaylandState {
    fn seat_state(&mut self) -> &mut SeatState {
        self.world_mut()
            .non_send_resource_mut::<SeatState>()
            .into_inner()
    }

    fn new_seat(
        &mut self,
        _conn: &smithay_client_toolkit::reexports::client::Connection,
        _qh: &QueueHandle<Self>,
        _seat: smithay_client_toolkit::reexports::client::protocol::wl_seat::WlSeat,
    ) {
    }

    fn new_capability(
        &mut self,
        _conn: &smithay_client_toolkit::reexports::client::Connection,
        qh: &QueueHandle<Self>,
        seat: smithay_client_toolkit::reexports::client::protocol::wl_seat::WlSeat,
        capability: smithay_client_toolkit::seat::Capability,
    ) {
        if capability == Capability::Keyboard {
            let mut seat_state = self.world_mut().non_send_resource_mut::<SeatState>();
            let wl_keyboard = seat_state
                .get_keyboard(qh, &seat, None)
                .expect("error while attaching keyboard!");
            self.world_mut().insert_non_send_resource(wl_keyboard);
            info!("Keyboard Attached");
        }
        if capability == Capability::Pointer {
            let mut seat_state = self.world_mut().non_send_resource_mut::<SeatState>();
            let wl_pointer = seat_state
                .get_pointer(qh, &seat)
                .expect("error while attaching pointer!");
            self.world_mut().insert_non_send_resource(wl_pointer);
            info!("Pointer Attached");
        }
        if capability == Capability::Touch {
            info!("Touchscreen Attached");
        }
    }

    fn remove_capability(
        &mut self,
        _conn: &smithay_client_toolkit::reexports::client::Connection,
        _qh: &QueueHandle<Self>,
        _seat: smithay_client_toolkit::reexports::client::protocol::wl_seat::WlSeat,
        capability: smithay_client_toolkit::seat::Capability,
    ) {
        if capability == Capability::Keyboard {
            self.world_mut().remove_non_send_resource::<WlKeyboard>();
            info!("Keyboard detatched");
        }
        if capability == Capability::Pointer {
            self.world_mut().remove_non_send_resource::<WlPointer>();
            info!("Pointer detatched");
        }
        if capability == Capability::Touch {
            info!("Touchscreen Attached");
        }
    }

    fn remove_seat(
        &mut self,
        _conn: &smithay_client_toolkit::reexports::client::Connection,
        _qh: &QueueHandle<Self>,
        _seat: smithay_client_toolkit::reexports::client::protocol::wl_seat::WlSeat,
    ) {
    }
}
delegate_seat!(WaylandState);
