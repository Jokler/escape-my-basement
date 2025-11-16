use bevy::prelude::*;
use bevy_ecs_ldtk::GridCoords;

pub fn plugin(app: &mut App) {
    app.add_systems(Update, translate_grid_coords_entities);
}

const GRID_SIZE: i32 = 16;

fn translate_grid_coords_entities(
    mut grid_coords_entities: Query<(&mut Transform, &GridCoords), Changed<GridCoords>>,
) {
    for (mut transform, grid_coords) in grid_coords_entities.iter_mut() {
        transform.translation =
            bevy_ecs_ldtk::utils::grid_coords_to_translation(*grid_coords, IVec2::splat(GRID_SIZE))
                .extend(transform.translation.z);
    }
}
