use bevy::prelude::*;

pub struct SettingsDrawerPlugin;
impl Plugin for SettingsDrawerPlugin {
    fn build(&self, app: &mut App) {}
}

pub mod prelude {
    pub use crate::SettingsDrawerPlugin;
}
