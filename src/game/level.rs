//! Spawn the main level.

use bevy::prelude::*;
use bevy_ecs_ldtk::{LdtkPlugin, LdtkWorldBundle, LevelSelection, app::LdtkEntityAppExt};

use crate::{
    asset_tracking::LoadResource, audio::music, game::player::PlayerSpawnBundle, screens::Screen,
};

mod walls;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(LdtkPlugin);
    app.insert_resource(LevelSelection::index(0));
    app.register_ldtk_entity::<PlayerSpawnBundle>("PlayerSpawn");
    app.load_resource::<LevelAssets>();

    app.add_plugins(walls::plugin);
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct LevelAssets {
    #[dependency]
    music: Handle<AudioSource>,
}

impl FromWorld for LevelAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            music: assets.load("audio/music/penis.ogg"),
        }
    }
}

/// A system that spawns the main level.
pub fn spawn_level(
    mut commands: Commands,
    level_assets: Res<LevelAssets>,
    asset_server: Res<AssetServer>,
) {
    commands.spawn((
        Name::new("Level"),
        Transform::default(),
        Visibility::default(),
        DespawnOnExit(Screen::Gameplay),
        children![
            LdtkWorldBundle {
                ldtk_handle: asset_server.load("levels.ldtk").into(),
                ..Default::default()
            },
            (
                Name::new("Gameplay Music"),
                music(level_assets.music.clone())
            )
        ],
    ));
}
