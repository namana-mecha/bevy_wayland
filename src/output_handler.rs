use bevy::prelude::*;
use smithay_client_toolkit::{
    delegate_output,
    output::{OutputHandler, OutputState},
    reexports::client::QueueHandle,
};

use crate::WaylandState;

pub struct OutputHandlerPlugin;
impl Plugin for OutputHandlerPlugin {
    fn build(&self, app: &mut App) {
        let globals = app.world().non_send_resource();
        let queue_handle: &QueueHandle<WaylandState> = app.world().non_send_resource();
        let output_state = OutputState::new(globals, queue_handle);

        app.insert_non_send_resource(output_state);
    }
}

impl OutputHandler for WaylandState {
    fn output_state(&mut self) -> &mut OutputState {
        self.world_mut()
            .non_send_resource_mut::<OutputState>()
            .into_inner()
    }

    fn new_output(
        &mut self,
        _conn: &smithay_client_toolkit::reexports::client::Connection,
        _qh: &QueueHandle<Self>,
        _output: smithay_client_toolkit::reexports::client::protocol::wl_output::WlOutput,
    ) {
        info!("new output was added");
    }

    fn update_output(
        &mut self,
        _conn: &smithay_client_toolkit::reexports::client::Connection,
        _qh: &QueueHandle<Self>,
        _output: smithay_client_toolkit::reexports::client::protocol::wl_output::WlOutput,
    ) {
    }

    fn output_destroyed(
        &mut self,
        _conn: &smithay_client_toolkit::reexports::client::Connection,
        _qh: &QueueHandle<Self>,
        _output: smithay_client_toolkit::reexports::client::protocol::wl_output::WlOutput,
    ) {
    }
}
delegate_output!(WaylandState);
