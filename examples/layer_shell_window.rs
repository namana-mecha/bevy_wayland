use std::time::Duration;

use bevy::{
    color::palettes::basic::*,
    prelude::*,
    window::{WindowCreated, WindowResolution},
    winit::WinitPlugin,
};
use bevy_wayland::{layer_shell::LayerShellSettings, WaylandPlugin};
use smithay_client_toolkit::shell::wlr_layer::{Anchor, Layer};

const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins
                .build()
                .disable::<WinitPlugin>()
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        resolution: WindowResolution::new(400.0, 400.0),
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
            WaylandPlugin,
        ))
        .init_resource::<NewWindowInfo>()
        .add_systems(Startup, setup)
        .add_systems(Update, (button_system, setup_new_window, spawn_window))
        .run();
}

#[allow(clippy::type_complexity)]
fn button_system(
    mut interaction_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            &mut BorderColor,
            &Children,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    mut text_query: Query<&mut Text>,
) {
    for (interaction, mut color, mut border_color, children) in &mut interaction_query {
        let mut text = text_query.get_mut(children[0]).unwrap();
        match *interaction {
            Interaction::Pressed => {
                **text = "Press".to_string();
                *color = PRESSED_BUTTON.into();
                border_color.0 = RED.into();
            }
            Interaction::Hovered => {
                **text = "Hover".to_string();
                *color = HOVERED_BUTTON.into();
                border_color.0 = Color::WHITE;
            }
            Interaction::None => {
                **text = "Button".to_string();
                *color = NORMAL_BUTTON.into();
                border_color.0 = Color::BLACK;
            }
        }
    }
}

#[derive(Component, Deref, DerefMut)]
struct WindowTimer(Timer);
fn setup(mut commands: Commands, assets: Res<AssetServer>, windows: Query<Entity, With<Window>>) {
    for entity in &windows {
        commands.entity(entity).insert((
            LayerShellSettings {
                anchor: Anchor::all(),
                layer: Layer::Bottom,
                ..Default::default()
            },
            WindowTimer(Timer::new(Duration::from_secs(5), TimerMode::Once)),
        ));
    }
    // ui camera
    commands.spawn(Camera2d);
    commands.spawn(button(&assets));
}

fn spawn_window(
    mut commands: Commands,
    mut windows: Query<(Entity, &mut WindowTimer)>,
    mut new_window_info: ResMut<NewWindowInfo>,
    time: Res<Time>,
) {
    for (entity, mut timer) in &mut windows {
        timer.tick(time.delta());

        if timer.finished() {
            commands.entity(entity).remove::<WindowTimer>();
            let new_window_entity = commands
                .spawn((
                    Window {
                        title: "UI Only Window".to_string(),
                        resolution: (400., 50.).into(),
                        ..default()
                    },
                    LayerShellSettings {
                        layer: Layer::Top,
                        anchor: Anchor::TOP | Anchor::LEFT,
                        ..default()
                    },
                    WindowTimer(Timer::new(Duration::from_secs(5), TimerMode::Once)),
                ))
                .id();

            new_window_info.entity = Some(new_window_entity);
            new_window_info.is_setup_pending = true;
        }
    }
}

#[derive(Resource, Default)]
struct NewWindowInfo {
    entity: Option<Entity>,
    is_setup_pending: bool,
}
fn setup_new_window(
    mut commands: Commands,
    mut window_created_events: EventReader<WindowCreated>,
    mut new_window_info: ResMut<NewWindowInfo>,
    asset_server: Res<AssetServer>, // For fonts
) {
    for event in window_created_events.read() {
        if Some(event.window) == new_window_info.entity && new_window_info.is_setup_pending {
            info!(
                "New UI window created (ID: {:?}), setting up its camera and UI.",
                event.window
            );

            commands.spawn((
                Camera {
                    target: bevy::render::camera::RenderTarget::Window(
                        bevy::window::WindowRef::Entity(event.window),
                    ),
                    clear_color: ClearColorConfig::Custom(Color::default()),
                    ..default()
                },
                Camera2d,
            ));
            new_window_info.is_setup_pending = false; // Mark as setup complete
        }
    }
}

fn button(asset_server: &AssetServer) -> impl Bundle + use<> {
    (
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        children![(
            Button,
            Node {
                width: Val::Px(150.0),
                height: Val::Px(65.0),
                border: UiRect::all(Val::Px(5.0)),
                // horizontally center child text
                justify_content: JustifyContent::Center,
                // vertically center child text
                align_items: AlignItems::Center,
                ..default()
            },
            BorderColor(Color::BLACK),
            BorderRadius::MAX,
            BackgroundColor(NORMAL_BUTTON),
            children![(
                Text::new("Button"),
                TextFont {
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    font_size: 33.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
                TextShadow::default(),
            )]
        )],
    )
}
