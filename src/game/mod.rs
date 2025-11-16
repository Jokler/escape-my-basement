//! Demo gameplay. All of these modules are only intended for demonstration
//! purposes and should be replaced with your own game logic.
//! Feel free to change the logic found here if you feel like tinkering around
//! to get a feeling for the template.

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
