use avian2d::prelude::{CollisionStart, RigidBody, Sensor};
use bevy::{
    ecs::{lifecycle::HookContext, world::DeferredWorld},
    prelude::*,
};
use bevy_ecs_ldtk::{EntityInstance, LdtkEntity, app::LdtkEntityAppExt, prelude::LdtkFields};

use crate::{
    game::{
        colliders::ColliderBundle,
        player::{Dead, Player},
    },
    menus::Menu,
};

pub fn plugin(app: &mut App) {
    app.register_ldtk_entity::<SpikeBundle>("Spike");
    app.add_systems(Update, spike_rotation);
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Component, Reflect)]
#[reflect(Component)]
#[component(on_add = on_spike_add)]
pub struct Spike;

pub fn on_spike_add(mut world: DeferredWorld, context: HookContext) {
    let spike_entity = context.entity;
    world
        .commands()
        .entity(spike_entity)
        .insert(Visibility::Hidden)
        .observe(on_player_touched_spike);
}

#[derive(Clone, Debug, Default, Bundle, LdtkEntity)]
pub struct SpikeBundle {
    spike: Spike,

    #[with(rotation_from_instance)]
    rotation: Rotation,

    #[sprite_sheet]
    sprite_sheet: Sprite,

    #[from_entity_instance]
    collider_bundle: ColliderBundle,

    sensor: Sensor,
}

#[derive(Clone, Debug, Default, Component, Reflect)]
pub enum Rotation {
    #[default]
    Bottom,
    Left,
    Top,
    Right,
}

pub fn spike_rotation(query: Query<(&mut Transform, &Rotation), Added<Rotation>>) {
    for (mut transform, rotation) in query {
        let degrees: f32 = match rotation {
            Rotation::Bottom => 0.,
            Rotation::Right => 90.,
            Rotation::Top => 180.,
            Rotation::Left => 270.,
        };
        *transform = transform.with_rotation(Quat::from_axis_angle(Vec3::Z, degrees.to_radians()));
    }
}

fn rotation_from_instance(instance: &EntityInstance) -> Rotation {
    match instance.get_enum_field("Rotation").map(|s| s.as_str()) {
        Ok("Top") => Rotation::Top,
        Ok("Left") => Rotation::Left,
        Ok("Right") => Rotation::Right,
        _ => Rotation::Bottom,
    }
}

fn on_player_touched_spike(
    event: On<CollisionStart>,
    mut commands: Commands,
    mut next_menu: ResMut<NextState<Menu>>,
    player_query: Query<Entity, With<Player>>,
) {
    // `colider1` and `body1` refer to the event target and its body.
    // `collider2` and `body2` refer to the other collider and its body.
    let spike_entity = event.collider1;
    let other_entity = event.collider2;

    for player_entity in player_query {
        if player_entity == other_entity {
            next_menu.set(Menu::Death);
            commands
                .entity(player_entity)
                .insert(Dead)
                .remove::<RigidBody>();
            commands.entity(spike_entity).insert(Visibility::Visible);
        }
    }
}
