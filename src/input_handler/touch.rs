use bevy::input::touch::{TouchInput, TouchPhase};
use bevy::log::{self, warn};
use bevy::math::Vec2;
use bevy::prelude::*;
use bevy::window::{Window, WindowEvent};
use smithay_client_toolkit::reexports::client::Proxy;
use smithay_client_toolkit::{
    delegate_touch,
    reexports::client::{
        protocol::{wl_surface::WlSurface, wl_touch::WlTouch},
        Connection, QueueHandle,
    },
    seat::touch::TouchHandler,
};

use crate::surface_handler::WaylandSurfaces;
use crate::WaylandState;

#[derive(Default, Resource, Deref, DerefMut)]
struct ActiveTouches(std::collections::HashMap<i32, (Entity, Vec2)>);

impl TouchHandler for WaylandState {
    /// Handles the "down" event when a touch point is pressed on the surface.
    fn down(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _touch: &WlTouch,
        _serial: u32,
        _time: u32,
        surface: WlSurface,
        id: i32,
        position: (f64, f64),
    ) {
        let window_entity = {
            let smithay_windows = self.world().non_send_resource::<WaylandSurfaces>();
            let window_id = surface.id();

            // Find the Bevy entity associated with the Smithay surface
            if let Some(entity) = smithay_windows.get_window_entity(&window_id).copied() {
                entity
            } else {
                warn!("touch down event on unknown surface: {:?}", window_id);
                return;
            }
        };

        let scale_factor = {
            // Get the window component to access scale factor
            if let Some(window) = self.world().get::<Window>(window_entity) {
                window.scale_factor()
            } else {
                warn!(
                    "touch down event for entity {:?} without a Window component",
                    window_entity
                );
                return;
            }
        };

        let logical_position = Vec2::new(position.0 as f32, position.1 as f32) / scale_factor;

        // Store the active touch point's entity and logical position
        let mut active_touches = self.world_mut().get_resource_or_init::<ActiveTouches>();
        active_touches.insert(id, (window_entity, logical_position));

        // Create and send the Bevy touch event
        let bevy_event = TouchInput {
            phase: TouchPhase::Started,
            position: logical_position,
            // Force is not directly available in basic Wayland touch events
            force: None,
            id: id as u64, // Bevy uses u64 for touch IDs
            window: window_entity,
        };

        // Send the event (adapt this line based on how you send events)
        self.world_mut().send_event(bevy_event);
        self.world_mut()
            .send_event(WindowEvent::TouchInput(bevy_event));
    }

    /// Handles the "up" event when a touch point is released from the surface.
    fn up(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _touch: &WlTouch,
        _serial: u32,
        _time: u32,
        id: i32,
    ) {
        // Retrieve the entity and last known position for the touch ID, then remove it
        let mut active_touches = self.world_mut().get_resource_or_init::<ActiveTouches>();
        let touch_data = active_touches.remove(&id);

        if let Some((entity, last_position)) = touch_data {
            // Create and send the Bevy touch event
            let bevy_event = TouchInput {
                phase: TouchPhase::Ended,
                position: last_position, // Use the stored last position
                force: None,
                id: id as u64,
                window: entity,
            };
            // Send the event
        self.world_mut().send_event(bevy_event);
        self.world_mut()
            .send_event(WindowEvent::TouchInput(bevy_event));
        } else {
            // This might happen if the 'down' event was missed or occurred on a different surface
            log::warn!(
                "touch up event for unknown or already removed touch ID: {}",
                id
            );
        }
    }

    /// Handles the "motion" event when a touch point is moved on the surface.
    fn motion(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _touch: &WlTouch,
        _time: u32,
        id: i32,
        position: (f64, f64),
    ) {
        let mut active_touches = self.world_mut().get_resource_or_init::<ActiveTouches>();
        // Get the entity associated with this ongoing touch ID
        let entity = if let Some((entity, _)) = active_touches.get(&id).copied() {
            entity
        } else {
            warn!("touch motion event for unknown touch ID: {}", id);
            return;
        };

        let scale_factor = {
            // Get the window component to access scale factor
            if let Some(window) = self.world().get::<Window>(entity) {
                window.scale_factor()
            } else {
                warn!(
                    "touch motion event for entity {:?} without a Window component",
                    entity
                );
                return;
            }
        };

        let logical_position = Vec2::new(position.0 as f32, position.1 as f32) / scale_factor;

        let mut active_touches = self.world_mut().get_resource_or_init::<ActiveTouches>();
        // Update the stored position for the touch ID
        if let Some(touch_data) = active_touches.get_mut(&id) {
            touch_data.1 = logical_position;
        } else {
            warn!("touch motion event occurred without a touch down event");
            return;
        }

        // Create and send the Bevy touch event
        let bevy_event = TouchInput {
            phase: TouchPhase::Moved,
            position: logical_position,
            force: None,
            id: id as u64,
            window: entity,
        };

        // Send the event
        self.world_mut().send_event(bevy_event);
        self.world_mut()
            .send_event(WindowEvent::TouchInput(bevy_event));
    }

    /// Handles the "cancel" event when a touch sequence is canceled.
    fn cancel(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _touch: &WlTouch) {
        let mut active_touches = self.world_mut().get_resource_or_init::<ActiveTouches>();
        let mut events = vec![];

        for id in active_touches.keys().copied().collect::<Vec<_>>() {
            let touch_data = active_touches.remove(&id);

            if let Some((entity, last_position)) = touch_data {
                // Create and send the Bevy touch event
                let bevy_event = TouchInput {
                    phase: TouchPhase::Canceled,
                    position: last_position, // Use the stored last position
                    force: None,
                    id: id as u64,
                    window: entity,
                };
                events.push(bevy_event)
            } else {
                warn!(
                    "touch cancel event for unknown or already removed touch ID: {}",
                    id
                );
                return;
            }
        }
        for bevy_event in events {
            self.world_mut().send_event(bevy_event);
            self.world_mut()
                .send_event(WindowEvent::TouchInput(bevy_event));
        }
    }

    /// Handles the "shape" event when the shape of a touch point changes.
    fn shape(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _touch: &WlTouch,
        _id: i32,
        _major: f64,
        _minor: f64,
    ) {
        // Handle touch shape change if needed
    }

    /// Handles the "orientation" event when the orientation of a touch point changes.
    fn orientation(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _touch: &WlTouch,
        _id: i32,
        _orientation: f64,
    ) {
        // Handle touch orientation change if needed
    }
}

// Delegate the touch handling implementation
delegate_touch!(WaylandState);
