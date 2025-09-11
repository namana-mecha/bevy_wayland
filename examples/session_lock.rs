use bevy::{
    prelude::*,
    window::{exit_on_all_closed, WindowRef},
    winit::WinitPlugin,
};
use bevy_wayland::prelude::*;
use smithay_client_toolkit::shell::wlr_layer::{Anchor, Layer};

const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins
                .build()
                .disable::<WinitPlugin>()
                .set(WindowPlugin {
                    primary_window: None,
                    ..Default::default()
                }),
            WaylandPlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                button_system,
                exit_on_esc,
                setup_session_lock_windows,
                exit_on_all_closed,
            ),
        )
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
        (Changed<Interaction>, With<UnlockButton>),
    >,
    mut text_query: Query<&mut Text>,
    mut session_lock_event_writer: EventWriter<SessionLockEvent>,
) {
    for (interaction, mut color, mut border_color, children) in &mut interaction_query {
        let mut text = text_query.get_mut(children[0]).unwrap();
        match *interaction {
            Interaction::Pressed => {
                session_lock_event_writer.write(SessionLockEvent::Unlock);
            }
            Interaction::Hovered => {
                **text = "Click to unlock".to_string();
                *color = HOVERED_BUTTON.into();
                border_color.0 = Color::WHITE;
            }
            Interaction::None => {
                **text = "Click to unlock".to_string();
                *color = NORMAL_BUTTON.into();
                border_color.0 = Color::BLACK;
            }
        }
    }
}

#[derive(Component)]
struct LockButton;
#[derive(Component)]
struct UnlockButton;

fn setup(
    mut commands: Commands,
    assets: Res<AssetServer>,
    windows: Query<Entity, With<Window>>,

    mut session_lock_event_writer: EventWriter<SessionLockEvent>,
) {
    session_lock_event_writer.write(SessionLockEvent::Lock);
    for entity in &windows {
        commands.entity(entity).insert((LayerShellSettings {
            anchor: Anchor::TOP | Anchor::LEFT,
            layer: Layer::Bottom,
            ..Default::default()
        },));
    }
    // ui camera
    commands.spawn(Camera2d);
    commands.spawn(lock_button(&assets));
}

#[derive(Component)]
struct ConfiguredWindow;
#[derive(Component)]
struct SessionLockCamera;

fn setup_session_lock_windows(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    windows: Query<(Entity, &SessionLockWindow), Without<ConfiguredWindow>>,
) {
    for (entity, _) in &windows {
        let camera = commands
            .spawn((
                Camera2d,
                Camera {
                    target: bevy::render::camera::RenderTarget::Window(WindowRef::Entity(entity)),
                    ..Default::default()
                },
                SessionLockCamera,
            ))
            .id();
        commands.entity(entity).insert(ConfiguredWindow);
        commands.spawn(unlock_button(&asset_server, camera));
    }
}

fn exit_on_esc(keys: Res<ButtonInput<KeyCode>>) {
    if keys.just_pressed(KeyCode::Escape) {
        std::process::exit(0);
    }
}

fn lock_button(asset_server: &AssetServer) -> impl Bundle + use<> {
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
            LockButton,
            Node {
                width: Val::Px(250.0),
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

fn unlock_button(asset_server: &AssetServer, camera: Entity) -> impl Bundle + use<> {
    (
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        UiTargetCamera(camera),
        children![(
            Button,
            UnlockButton,
            Node {
                width: Val::Px(250.0),
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
