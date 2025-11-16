use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;

use avian2d::prelude::*;

use crate::{
    AppSystems,
    game::mine::{Mine, on_player_touched_mine},
};

pub fn plugin(app: &mut App) {
    app.add_systems(Update, add_child_colliders.in_set(AppSystems::Update));
}

#[derive(Clone, Debug, Default, Bundle, LdtkIntCell)]
pub struct ColliderBundle {
    pub collider: Collider,
    pub rigid_body: RigidBody,
    pub velocity: LinearVelocity,
    pub rotation_constraints: LockedAxes,
    pub gravity_scale: GravityScale,
    pub friction: Friction,
    pub density: ColliderDensity,
    pub events: CollisionEventsEnabled,
}

impl From<&EntityInstance> for ColliderBundle {
    fn from(entity_instance: &EntityInstance) -> ColliderBundle {
        let rotation_constraints = LockedAxes::ROTATION_LOCKED;

        match entity_instance.identifier.as_ref() {
            "Spike" => ColliderBundle {
                collider: Collider::rectangle(10., 10.),
                rigid_body: RigidBody::Kinematic,
                rotation_constraints,
                ..Default::default()
            },
            "Mine" => ColliderBundle {
                collider: Collider::rectangle(16., 16.),
                rigid_body: RigidBody::Kinematic,
                rotation_constraints,
                ..Default::default()
            },
            _ => ColliderBundle::default(),
        }
    }
}

#[derive(Debug, Component, Reflect, Clone, Copy)]
pub struct ColliderInserted;

pub fn add_child_colliders(
    mut commands: Commands,
    collider_query: Query<Entity, (With<Mine>, Without<ColliderInserted>)>,
) {
    for collider_entity in collider_query {
        commands
            .entity(collider_entity)
            .remove::<Collider>()
            .insert(ColliderInserted)
            .with_children(|c| {
                c.spawn((
                    CollisionEventsEnabled,
                    Transform::from_translation(Vec3::new(0., -6.5, 0.)),
                    Collider::rectangle(16., 3.),
                    Sensor,
                ))
                .observe(on_player_touched_mine);
            });
    }
}
