use bevy::{prelude::*, winit::WinitPlugin};
use bevy_wayland::WaylandPlugin;
use mechanix_launcher::LauncherPlugin;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins
                .build()
                .disable::<WinitPlugin>()
                .set(WindowPlugin {
                    primary_window: None,
                    exit_condition: bevy::window::ExitCondition::DontExit,
                    ..Default::default()
                }),
            LauncherPlugin,
            WaylandPlugin,
        ))
        .run();
}
