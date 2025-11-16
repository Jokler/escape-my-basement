//! Player-specific behavior.

use std::time::Duration;

use avian2d::prelude::{Collider, CollisionEventsEnabled, Friction, LockedAxes, RigidBody};
use bevy::{
    ecs::{lifecycle::HookContext, world::DeferredWorld},
    image::{ImageLoaderSettings, ImageSampler},
    prelude::*,
};
use bevy_ecs_ldtk::LdtkEntity;
use bevy_tnua::{
    TnuaUserControlsSystems,
    prelude::{TnuaBuiltinJump, TnuaBuiltinWalk, TnuaController},
};
use bevy_tnua_avian2d::TnuaAvian2dSensorShape;
use rand::seq::IndexedRandom;

use crate::{
    AppSystems, PausableSystems,
    asset_tracking::LoadResource,
    audio::sound_effect,
    follow_camera,
    game::animation::{Animation, AnimationData, AnimationState, Repeat},
};

pub(super) fn plugin(app: &mut App) {
    app.load_resource::<PlayerAssets>();

    // Record directional input as movement controls.
    app.add_systems(
        Update,
        (follow_camera)
            .in_set(AppSystems::RecordInput)
            .in_set(PausableSystems),
    );
    app.add_systems(FixedUpdate, apply_controls.in_set(TnuaUserControlsSystems));
    app.add_systems(Update, despawn_player.in_set(AppSystems::Update));

    app.add_observer(on_spawn_player);
}

#[derive(Default, Bundle, LdtkEntity)]
pub struct PlayerSpawnBundle {
    player: PlayerSpawn,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
#[component(on_add = on_player_spawn_add)]
pub struct PlayerSpawn;

pub fn on_player_spawn_add(mut world: DeferredWorld, context: HookContext) {
    let spawner_entity = context.entity;
    world.trigger(SpawnPlayer(spawner_entity));
}

#[derive(Event)]
pub struct SpawnPlayer(pub Entity);

fn on_spawn_player(
    event: On<SpawnPlayer>,
    mut commands: Commands,
    player_assets: Res<PlayerAssets>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    players: Query<(), (With<Player>, Without<Dead>)>,
) {
    if players.is_empty() {
        commands.entity(event.event().0).with_children(|p| {
            p.spawn(player(&player_assets, &mut texture_atlas_layouts));
        });
    }
}

/// The player character.
pub fn player(
    player_assets: &PlayerAssets,
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
) -> impl Bundle {
    let run = AnimationData {
        frames: 6,
        interval: Duration::from_millis(80),
        state: AnimationState::Walking,
        atlas_index: 0,
        repeat: Repeat::Loop,
    };
    let idle = AnimationData {
        frames: 4,
        interval: Duration::from_millis(150),
        state: AnimationState::Idle,
        atlas_index: 6,
        repeat: Repeat::Loop,
    };
    let fall = AnimationData {
        frames: 3,
        interval: Duration::from_millis(150),
        state: AnimationState::Falling,
        atlas_index: 10,
        repeat: Repeat::Loop,
    };
    let jump = AnimationData {
        frames: 3,
        interval: Duration::from_millis(150),
        state: AnimationState::Jumping,
        atlas_index: 13,
        repeat: Repeat::Loop,
    };
    let death = AnimationData {
        frames: 3,
        interval: Duration::from_millis(80),
        state: AnimationState::Dying,
        atlas_index: 16,
        repeat: Repeat::OneShot,
    };

    // A texture atlas is a way to split a single image into a grid of related images.
    // You can learn more in this example: https://github.com/bevyengine/bevy/blob/latest/examples/2d/texture_atlas.rs
    let layout = TextureAtlasLayout::from_grid(UVec2::splat(16), 4, 5, None, None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);
    let player_animation = Animation::new(vec![run, idle, fall, jump, death]);

    (
        Player,
        Name::new("Player"),
        Sprite::from_atlas_image(
            player_assets.ducky.clone(),
            TextureAtlas {
                layout: texture_atlas_layout,
                index: player_animation.get_atlas_index(),
            },
        ),
        player_animation,
        // The player character needs to be configured as a dynamic rigid body of the physics
        // engine.
        RigidBody::Dynamic,
        Collider::round_rectangle(8.0, 8.0, 1.0),
        // This is Tnua's interface component.
        TnuaController::default(),
        // A sensor shape is not strictly necessary, but without it we'll get weird results.
        TnuaAvian2dSensorShape(Collider::rectangle(8., 8.)),
        // Tnua can fix the rotation, but the character will still get rotated before it can do so.
        // By locking the rotation we can prevent this.
        LockedAxes::ROTATION_LOCKED,
        CollisionEventsEnabled,
        Friction::new(0.0),
    )
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
pub struct Player;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(Component)]
pub struct Dead;

fn apply_controls(
    mut just_jumped: Local<bool>,
    mut commands: Commands,
    player_assets: If<Res<PlayerAssets>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut TnuaController, &mut Sprite)>,
) {
    let Ok((mut controller, mut sprite)) = query.single_mut() else {
        return;
    };

    let mut direction = Vec3::ZERO;

    if keyboard.pressed(KeyCode::KeyR) || keyboard.pressed(KeyCode::KeyA) {
        direction -= Vec3::X;
    }
    if keyboard.pressed(KeyCode::KeyT) || keyboard.pressed(KeyCode::KeyD) {
        direction += Vec3::X;
    }

    if direction.x != 0.0 {
        sprite.flip_x = direction.x < 0.0;
    }

    // Feed the basis every frame. Even if the player doesn't move - just use `desired_velocity:
    // Vec3::ZERO`. `TnuaController` starts without a basis, which will make the character collider
    // just fall.
    controller.basis(TnuaBuiltinWalk {
        // The `desired_velocity` determines how the character will move.
        desired_velocity: direction.normalize_or_zero() * 120.0,
        acceleration: 800.0,
        air_acceleration: 400.0,
        // The `float_height` must be greater (even if by little) from the distance between the
        // character's center and the lowest point of its collider.
        float_height: 1.5,
        // `TnuaBuiltinWalk` has many other fields for customizing the movement - but they have
        // sensible defaults. Refer to the `TnuaBuiltinWalk`'s documentation to learn what they do.
        ..Default::default()
    });

    // Feed the jump action every frame as long as the player holds the jump button. If the player
    // stops holding the jump button, simply stop feeding the action.
    if keyboard.pressed(KeyCode::Space) {
        controller.action(TnuaBuiltinJump {
            // The height is the only mandatory field of the jump button.
            height: 35.0,
            // `TnuaBuiltinJump` also has customization fields with sensible defaults.
            ..Default::default()
        });
        if !controller.is_airborne().unwrap_or(true) {
            if !*just_jumped {
                let rng = &mut rand::rng();
                let random_step = player_assets.jumps.choose(rng).unwrap().clone();
                commands.spawn((Name::new("Walking Sound"), sound_effect(random_step)));
                *just_jumped = true;
            }
        } else {
            *just_jumped = false;
        }
    }
}

pub fn despawn_player(mut commands: Commands, explosions: Query<(Entity, &Animation), With<Dead>>) {
    for (entity, animation) in explosions {
        if animation.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct PlayerAssets {
    #[dependency]
    ducky: Handle<Image>,
    #[dependency]
    pub jumps: Vec<Handle<AudioSource>>,
    #[dependency]
    pub death: Handle<AudioSource>,
}

impl FromWorld for PlayerAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            ducky: assets.load_with_settings(
                "images/hero.png",
                |settings: &mut ImageLoaderSettings| {
                    settings.sampler = ImageSampler::nearest();
                },
            ),
            jumps: vec![assets.load("audio/sound_effects/jump.ogg")],
            death: assets.load("audio/sound_effects/death.ogg"),
        }
    }
}
