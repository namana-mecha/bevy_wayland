use bevy::{prelude::*, winit::WinitPlugin};
use bevy_wayland::{layer_shell::LayerShellWindow, WaylandPlugin};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.build().disable::<WinitPlugin>(),
            WaylandPlugin,
        ))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands, windows: Query<Entity, With<Window>>) {
    for entity in &windows {
        commands.entity(entity).insert(LayerShellWindow::default());
    }
}
