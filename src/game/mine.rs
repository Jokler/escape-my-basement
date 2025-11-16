use std::time::Duration;

use avian2d::prelude::{CollisionStart, RigidBody, Sensor};
use bevy::{
    ecs::{lifecycle::HookContext, world::DeferredWorld},
    image::{ImageLoaderSettings, ImageSampler},
    prelude::*,
};
use bevy_ecs_ldtk::{LdtkEntity, app::LdtkEntityAppExt};
use rand::seq::IndexedRandom;

use crate::{
    AppSystems,
    asset_tracking::LoadResource,
    audio::sound_effect,
    game::{
        animation::{Animation, AnimationData, AnimationState, Repeat},
        colliders::ColliderBundle,
        player::{Dead, Player},
    },
    menus::Menu,
};

pub fn plugin(app: &mut App) {
    app.load_resource::<MineAssets>();
    app.register_ldtk_entity::<MineBundle>("Mine");
    app.add_systems(Update, despawn_explosion.in_set(AppSystems::Update));
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Component, Reflect)]
#[reflect(Component)]
#[component(on_add = on_mine_add)]
pub struct Mine;

pub fn on_mine_add(mut world: DeferredWorld, context: HookContext) {
    let mine_entity = context.entity;
    world
        .commands()
        .entity(mine_entity)
        .insert(Visibility::Hidden);
}

#[derive(Clone, Debug, Default, Bundle, LdtkEntity)]
pub struct MineBundle {
    mine: Mine,

    #[sprite_sheet]
    sprite_sheet: Sprite,

    #[from_entity_instance]
    collider_bundle: ColliderBundle,

    sensor: Sensor,
}

pub fn on_player_touched_mine(
    event: On<CollisionStart>,
    mut commands: Commands,
    mut next_menu: ResMut<NextState<Menu>>,
    player_query: Query<Entity, With<Player>>,
    mine_assets: Res<MineAssets>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    parents: Query<&ChildOf>,
    transforms: Query<&Transform>,
) {
    let mine_entity = parents.get(event.collider1).unwrap().0;
    let other_entity = event.collider2;

    for player_entity in player_query {
        if player_entity == other_entity {
            next_menu.set(Menu::Death);
            commands
                .entity(player_entity)
                .insert(Dead)
                .remove::<RigidBody>();

            let mine_transform = transforms.get(mine_entity).unwrap();
            let mut transform = *mine_transform;
            transform.translation.y += 6.5;

            commands.entity(mine_entity).insert((
                transform,
                explosion(&mine_assets, &mut texture_atlas_layouts),
                Visibility::Visible,
            ));

            let rng = &mut rand::rng();
            let random_boom = mine_assets.booms.choose(rng).unwrap().clone();

            commands.spawn((Name::from("Boom Sound"), sound_effect(random_boom)));
        }
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
pub struct Explosion;

pub fn explosion(
    mine_assets: &MineAssets,
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
) -> impl Bundle {
    let explode = AnimationData {
        frames: 8,
        interval: Duration::from_millis(70),
        state: AnimationState::Idle,
        atlas_index: 0,
        repeat: Repeat::OneShot,
    };

    let layout = TextureAtlasLayout::from_grid(UVec2::splat(32), 8, 1, None, None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);
    let explode_animation = Animation::new(vec![explode]);

    (
        Explosion,
        Name::new("Explosion"),
        Sprite::from_atlas_image(
            mine_assets.explosion.clone(),
            TextureAtlas {
                layout: texture_atlas_layout,
                index: explode_animation.get_atlas_index(),
            },
        ),
        explode_animation,
    )
}

pub fn despawn_explosion(
    mut commands: Commands,
    explosions: Query<(Entity, &Animation), (With<Explosion>, Without<AudioSink>)>,
) {
    for (entity, animation) in explosions {
        if animation.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct MineAssets {
    #[dependency]
    explosion: Handle<Image>,
    #[dependency]
    pub booms: Vec<Handle<AudioSource>>,
}

impl FromWorld for MineAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            explosion: assets.load_with_settings(
                "images/boom.png",
                |settings: &mut ImageLoaderSettings| {
                    settings.sampler = ImageSampler::nearest();
                },
            ),
            booms: vec![assets.load("audio/sound_effects/boom.ogg")],
        }
    }
}
