use bevy::prelude::*;

mod animation;
mod colliders;
mod door;
mod grid_coords;
pub mod level;
mod mine;
mod physics;
pub mod player;
mod spike;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        animation::plugin,
        level::plugin,
        player::plugin,
        physics::plugin,
        grid_coords::plugin,
        door::plugin,
        spike::plugin,
        mine::plugin,
        colliders::plugin,
    ));
}
