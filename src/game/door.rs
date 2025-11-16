use avian2d::prelude::{
    Collider, CollisionEventsEnabled, CollisionStart, Friction, RigidBody, Sensor,
};
use bevy::{
    platform::collections::{HashMap, HashSet},
    prelude::*,
};
use bevy_ecs_ldtk::{
    GridCoords, LdtkIntCell, LdtkProjectHandle, LevelIid, LevelSelection, app::LdtkIntCellAppExt,
    assets::LdtkProject, ldtk::LayerInstance,
};

use crate::{game::player::Player, menus::Menu};

pub fn plugin(app: &mut App) {
    app.add_systems(Update, spawn_door_sensor)
        .register_ldtk_int_cell::<DoorBundle>(3);
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Component)]
pub struct Door;

#[derive(Clone, Debug, Default, Bundle, LdtkIntCell)]
pub struct DoorBundle {
    door: Door,
}

fn on_player_entered_door(
    event: On<CollisionStart>,
    player_query: Query<&Player>,
    level_selection: ResMut<LevelSelection>,
    mut next_menu: ResMut<NextState<Menu>>,
) {
    // `colider1` and `body1` refer to the event target and its body.
    // `collider2` and `body2` refer to the other collider and its body.
    let other_entity = event.collider2;

    if player_query.contains(other_entity) {
        let indices = match level_selection.into_inner() {
            LevelSelection::Indices(indices) => indices,
            _ => panic!("level selection should always be Indices in this game"),
        };

        indices.level += 1;

        if indices.level > 4 {
            next_menu.set(Menu::Won);
        }
    }
}

pub fn spawn_door_sensor(
    mut commands: Commands,
    door_query: Query<(&GridCoords, &ChildOf), Added<Door>>,
    parent_query: Query<&ChildOf, Without<Door>>,
    level_query: Query<(Entity, &LevelIid)>,
    ldtk_projects: Query<&LdtkProjectHandle>,
    ldtk_project_assets: Res<Assets<LdtkProject>>,
) {
    /// Represents a wide door that is 1 tile tall
    /// Used to spawn door collisions
    #[derive(Clone, Eq, PartialEq, Debug, Default, Hash)]
    struct Plate {
        left: i32,
        right: i32,
    }

    /// A simple rectangle type representing a door of any size
    struct Rect {
        left: i32,
        right: i32,
        top: i32,
        bottom: i32,
    }

    // Consider where the doors are
    // storing them as GridCoords in a HashSet for quick, easy lookup
    //
    // The key of this map will be the entity of the level the door belongs to.
    // This has two consequences in the resulting collision entities:
    // 1. it forces the doors to be split along level boundaries
    // 2. it lets us easily add the collision entities as children of the appropriate level entity
    let mut level_to_door_locations: HashMap<Entity, HashSet<GridCoords>> = HashMap::new();

    door_query.iter().for_each(|(&grid_coords, parent)| {
        // An intgrid tile's direct parent will be a layer entity, not the level entity
        // To get the level entity, you need the tile's grandparent.
        // This is where parent_query comes in.
        if let Ok(grandparent) = parent_query.get(parent.parent()) {
            level_to_door_locations
                .entry(grandparent.parent())
                .or_default()
                .insert(grid_coords);
        }
    });

    if !door_query.is_empty() {
        level_query.iter().for_each(|(level_entity, level_iid)| {
            if let Some(level_doors) = level_to_door_locations.get(&level_entity) {
                let ldtk_project = ldtk_project_assets
                    .get(ldtk_projects.single().unwrap())
                    .expect("Project should be loaded if level has spawned");

                let level = ldtk_project
                    .as_standalone()
                    .get_loaded_level_by_iid(&level_iid.to_string())
                    .expect("Spawned level should exist in LDtk project");

                let LayerInstance {
                    c_wid: width,
                    c_hei: height,
                    grid_size,
                    ..
                } = level.layer_instances()[0];

                // combine door tiles into flat "plates" in each individual row
                let mut plate_stack: Vec<Vec<Plate>> = Vec::new();

                for y in 0..height {
                    let mut row_plates: Vec<Plate> = Vec::new();
                    let mut plate_start = None;

                    // + 1 to the width so the algorithm "terminates" plates that touch the right edge
                    for x in 0..=width {
                        match (plate_start, level_doors.contains(&GridCoords { x, y })) {
                            (Some(s), false) => {
                                row_plates.push(Plate {
                                    left: s,
                                    right: x - 1,
                                });
                                plate_start = None;
                            }
                            (None, true) => plate_start = Some(x),
                            _ => (),
                        }
                    }

                    plate_stack.push(row_plates);
                }

                // combine "plates" into rectangles across multiple rows
                let mut rect_builder: HashMap<Plate, Rect> = HashMap::new();
                let mut prev_row: Vec<Plate> = Vec::new();
                let mut door_rects: Vec<Rect> = Vec::new();

                // an extra empty row so the algorithm "finishes" the rects that touch the top edge
                plate_stack.push(Vec::new());

                for (y, current_row) in plate_stack.into_iter().enumerate() {
                    for prev_plate in &prev_row {
                        if !current_row.contains(prev_plate) {
                            // remove the finished rect so that the same plate in the future starts a new rect
                            if let Some(rect) = rect_builder.remove(prev_plate) {
                                door_rects.push(rect);
                            }
                        }
                    }
                    for plate in &current_row {
                        rect_builder
                            .entry(plate.clone())
                            .and_modify(|e| e.top += 1)
                            .or_insert(Rect {
                                bottom: y as i32,
                                top: y as i32,
                                left: plate.left,
                                right: plate.right,
                            });
                    }
                    prev_row = current_row;
                }

                commands.entity(level_entity).with_children(|level| {
                    // Spawn colliders for every rectangle..
                    // Making the collider a child of the level serves two purposes:
                    // 1. Adjusts the transforms to be relative to the level for free
                    // 2. the colliders will be despawned automatically when levels unload
                    for door_rect in door_rects {
                        let width = (door_rect.right as f32 - door_rect.left as f32 + 1.)
                            * grid_size as f32;
                        let height = (door_rect.top as f32 - door_rect.bottom as f32 + 1.)
                            * grid_size as f32;
                        level
                            .spawn((
                                Collider::rectangle(width, height),
                                Sensor,
                                RigidBody::Static,
                                Transform::from_xyz(
                                    (door_rect.left + door_rect.right + 1) as f32
                                        * grid_size as f32
                                        / 2.,
                                    (door_rect.bottom + door_rect.top + 1) as f32
                                        * grid_size as f32
                                        / 2.,
                                    0.,
                                ),
                                GlobalTransform::default(),
                                InheritedVisibility::default(),
                                Name::new("Door"),
                                Door,
                                CollisionEventsEnabled,
                            ))
                            .observe(on_player_entered_door);
                    }
                });
            }
        });
    }
}
