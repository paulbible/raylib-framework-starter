//! Structs used for creating multple scenes.
//! 
//! 
pub mod game_data;
pub mod scenes;
pub mod game_scene;
pub mod menu_scene;
pub mod maze_scene;
pub mod utils;

pub fn is_floor_tile(tile_id: i32) -> bool {
    matches!(
        tile_id,
        4..=81 | 191..=218
    )
}

pub fn is_wall_tile(tile_id: i32) -> bool {
    matches!(
        tile_id,
        88..=190
    )
}

