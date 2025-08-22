use bevy::{
    input::{
        mouse::{MouseButtonInput, MouseScrollUnit, MouseWheel},
        ButtonState,
    },
    prelude::MouseButton,
    window::{CursorEntered, CursorLeft, CursorMoved, Window, WindowEvent},
};
use smithay_client_toolkit::{
    delegate_pointer,
    reexports::{client::Proxy, csd_frame::WindowState},
    seat::pointer::PointerHandler,
};

use crate::{surface_handler::WaylandSurfaces, WaylandState};

/// Converts a u32 button code to a Bevy MouseButton.
fn convert_to_mouse_button(button: u32) -> MouseButton {
    match button {
        272 => MouseButton::Left,
        273 => MouseButton::Right,
        274 => MouseButton::Middle,
        277 => MouseButton::Forward,
        278 => MouseButton::Back,
        other => MouseButton::Other(other as u16),
    }
}

impl PointerHandler for WaylandState {
    fn pointer_frame(
        &mut self,
        _: &smithay_client_toolkit::reexports::client::Connection,
        _: &smithay_client_toolkit::reexports::client::QueueHandle<Self>,
        _: &smithay_client_toolkit::reexports::client::protocol::wl_pointer::WlPointer,
        events: &[smithay_client_toolkit::seat::pointer::PointerEvent],
    ) {
        for event in events {
            let smithay_windows = self.world().non_send_resource::<WaylandSurfaces>();
            let entity = smithay_windows.get_window_entity(&event.surface.id());

            if entity.is_none() {
                continue;
            }
            let entity = *entity.unwrap();

            let window = self.world().get::<Window>(entity).unwrap().clone();
            let mut position = bevy::math::Vec2 {
                x: event.position.0 as f32,
                y: event.position.1 as f32,
            };
            let delta = window
                .physical_cursor_position()
                .map(|old_position| (position - old_position) / window.scale_factor());
            let pointer_event: WindowEvent = match event.kind {
                smithay_client_toolkit::seat::pointer::PointerEventKind::Enter { .. } => {
                    CursorEntered { window: entity }.into()
                }
                smithay_client_toolkit::seat::pointer::PointerEventKind::Leave { .. } => {
                    CursorLeft { window: entity }.into()
                }
                smithay_client_toolkit::seat::pointer::PointerEventKind::Motion { .. } => {
                    self.world_mut()
                        .get_mut::<Window>(entity)
                        .unwrap()
                        .set_physical_cursor_position(Some(position.as_dvec2()));
                    position /= window.scale_factor();
                    CursorMoved {
                        window: entity,
                        position,
                        delta,
                    }
                    .into()
                }
                smithay_client_toolkit::seat::pointer::PointerEventKind::Press {
                    button, ..
                } => MouseButtonInput {
                    button: convert_to_mouse_button(button),
                    state: ButtonState::Pressed,
                    window: entity,
                }
                .into(),

                smithay_client_toolkit::seat::pointer::PointerEventKind::Release {
                    button, ..
                } => MouseButtonInput {
                    button: convert_to_mouse_button(button),
                    state: ButtonState::Released,
                    window: entity,
                }
                .into(),
                smithay_client_toolkit::seat::pointer::PointerEventKind::Axis {
                    horizontal,
                    vertical,
                    ..
                } => MouseWheel {
                    unit: MouseScrollUnit::Pixel,
                    x: horizontal.absolute as f32,
                    y: vertical.absolute as f32,
                    window: entity,
                }
                .into(),
            };
            let window_event: WindowEvent = pointer_event;
            self.world_mut().send_event::<WindowEvent>(window_event);
        }
    }
}
delegate_pointer!(WaylandState);
