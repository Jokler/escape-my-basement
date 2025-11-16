//! The won menu.

use bevy::prelude::*;
use bevy_ecs_ldtk::LdtkProjectHandle;

use crate::{menus::Menu, screens::Screen, theme::widget};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Menu::Won), spawn_won_menu);
}

fn spawn_won_menu(mut commands: Commands, ldtk_projects: Query<Entity, With<LdtkProjectHandle>>) {
    commands.entity(ldtk_projects.single().unwrap()).despawn();

    commands.spawn((
        widget::ui_root("Won Menu"),
        GlobalZIndex(2),
        DespawnOnExit(Menu::Won),
        children![
            widget::header("You Win!"),
            widget::button("Quit to title", quit_to_title),
        ],
    ));
}

fn quit_to_title(_: On<Pointer<Click>>, mut next_screen: ResMut<NextState<Screen>>) {
    next_screen.set(Screen::Title);
}
