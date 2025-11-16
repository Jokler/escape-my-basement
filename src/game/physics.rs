use avian2d::{
    PhysicsPlugins,
    prelude::{Gravity, PhysicsDebugPlugin},
};
use bevy::{
    app::{App, FixedUpdate},
    math::Vec2,
};
use bevy_tnua::prelude::TnuaControllerPlugin;
use bevy_tnua_avian2d::TnuaAvian2dPlugin;

pub fn plugin(app: &mut App) {
    app.add_plugins((
        PhysicsPlugins::default(),
        TnuaControllerPlugin::new(FixedUpdate),
        TnuaAvian2dPlugin::new(FixedUpdate),
        // PhysicsDebugPlugin,
    ))
    .insert_resource(Gravity(Vec2::NEG_Y * 300.0));
}
