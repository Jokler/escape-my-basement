//! The death menu.

use std::time::Duration;

use bevy::{input::common_conditions::input_just_pressed, prelude::*};

use crate::{
    game::player::{PlayerSpawn, SpawnPlayer},
    menus::Menu,
    screens::Screen,
    theme::widget,
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Menu::Death), spawn_death_menu);
    app.add_systems(
        Update,
        (
            make_visible,
            go_back.run_if(in_state(Menu::Death).and(input_just_pressed(KeyCode::KeyR))),
        ),
    );
}

#[derive(Clone, Copy, Debug, Component, Reflect)]
struct VisibleAt(Duration);

fn spawn_death_menu(mut commands: Commands, time: Res<Time>) {
    commands.spawn((
        Visibility::Hidden,
        VisibleAt(time.elapsed() + Duration::from_millis(500)),
        widget::ui_root("Death Menu"),
        GlobalZIndex(2),
        DespawnOnExit(Menu::Death),
        children![
            widget::header("You Died!"),
            widget::button("Restart", restart),
            widget::button("Quit to title", quit_to_title),
        ],
    ));
}

fn make_visible(
    mut commands: Commands,
    entity_query: Query<(Entity, &VisibleAt)>,
    time: Res<Time>,
) {
    for (entity, visible_at) in entity_query {
        if visible_at.0 <= time.elapsed() {
            commands.entity(entity).insert(Visibility::Visible);
        }
    }
}

fn restart(
    _: On<Pointer<Click>>,
    mut commands: Commands,
    player_spawner_entity: Single<Entity, With<PlayerSpawn>>,
    mut next_menu: ResMut<NextState<Menu>>,
) -> Result {
    commands.trigger(SpawnPlayer(player_spawner_entity.entity()));
    next_menu.set(Menu::None);

    Ok(())
}

fn quit_to_title(_: On<Pointer<Click>>, mut next_screen: ResMut<NextState<Screen>>) {
    next_screen.set(Screen::Title);
}

fn go_back(
    mut commands: Commands,
    player_spawner_entity: Single<Entity, With<PlayerSpawn>>,
    mut next_menu: ResMut<NextState<Menu>>,
) {
    commands.trigger(SpawnPlayer(player_spawner_entity.entity()));
    next_menu.set(Menu::None);
}
