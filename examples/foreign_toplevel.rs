use bevy::{color::palettes::basic::*, prelude::*, window::WindowResolution, winit::WinitPlugin};
use bevy_wayland::{foreign_toplevel_manager::ForeignToplevelEvent, prelude::*};

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
                        present_mode: bevy::window::PresentMode::AutoVsync,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
            WaylandPlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, (button_system, exit_on_esc))
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
    mut foreign_toplevel_event_writer: EventWriter<ForeignToplevelEvent>,
    mut text_query: Query<&mut Text>,
) {
    //info!("Button system was called!!");
    for (interaction, mut color, mut border_color, children) in &mut interaction_query {
        let mut text = text_query.get_mut(children[0]).unwrap();
        match *interaction {
            Interaction::Pressed => {
                **text = "Press".to_string();
                *color = PRESSED_BUTTON.into();
                border_color.0 = RED.into();
                foreign_toplevel_event_writer.write(ForeignToplevelEvent::MinimizeOthers);
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

fn setup(mut commands: Commands, assets: Res<AssetServer>, windows: Query<Entity, With<Window>>) {
    for entity in &windows {
        commands.entity(entity).insert((LayerShellSettings {
            anchor: Anchor::TOP | Anchor::LEFT,
            layer: Layer::Top,
            ..Default::default()
        },));
    }
    // ui camera
    commands.spawn(Camera2d);
    commands.spawn(button(&assets));
}

fn exit_on_esc(keys: Res<ButtonInput<KeyCode>>) {
    if keys.just_pressed(KeyCode::Escape) {
        std::process::exit(0);
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
