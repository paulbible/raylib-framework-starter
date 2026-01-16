use raylib::prelude::*;

use crate::menu_scene::WinScene;
use crate::scenes::{Scene, SceneSwitch};
use crate::game_data::GameData;
use crate::{is_floor_tile, is_wall_tile};
use std::fs::File;
use std::io::Read;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct MapData {
    pub grid_w: usize,
    pub grid_h: usize,
    pub tile_size_px: i32,
    pub tiles: Vec<Vec<i32>>,
    pub entities: Vec<MapEntity>,
}

#[derive(Deserialize)]
pub struct MapEntity {
    pub kind: String,
    pub x: usize,
    pub y: usize,
}


pub fn load_map(path: &str) -> MapData {
    let mut file = File::open(path).expect("Failed to open map.json");
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    serde_json::from_str(&contents).expect("Invalid map.json")
}

pub struct MazeScene {
    pub map_path: String,   

    map: MapData,

    tileset: Option<Texture2D>, // Use Option since we can't load it in from_map
    tile_size: i32,

    player_x: usize,
    player_y: usize,

    player_speed: f32,
}





impl MazeScene {
    pub fn from_map(path: String) -> Self {
        Self {
            map_path: path.clone(),
            map: load_map(&path),
            tileset: None, 
            tile_size: 32,
            player_x: 0,
            player_y: 0,
            player_speed: 0.0,
        }
    }


    
    
    
    // Check if a move to the given position is valid
    fn is_valid_move(&self, x: usize, y: usize) -> bool {
        if x >= self.map.grid_w || y >= self.map.grid_h {
            return false;
        }
        let tid = self.map.tiles[y][x];
        tid >= 0 && !is_wall_tile(tid)
    }
    fn draw_tile(&self, d: &mut RaylibDrawHandle, tile_id: i32, x: usize, y: usize) {
        let tileset = match &self.tileset {
            Some(t) => t,
            None => return, 
        };
        let cols = tileset.width() / self.tile_size;
        let src = Rectangle {
            x: ((tile_id % cols) * self.tile_size) as f32,
            y: ((tile_id / cols) * self.tile_size) as f32,
            width: self.tile_size as f32,
            height: self.tile_size as f32,
        };

        let dst = Rectangle {
            x: (x as i32 * self.tile_size) as f32,
            y: (y as i32 * self.tile_size) as f32,
            width: self.tile_size as f32,
            height: self.tile_size as f32,
        };

        d.draw_texture_pro(tileset, 
            src, 
            dst, Vector2::zero(), 0.0, Color::WHITE);
    }

    fn tile_src_rect(tile_id: i32, tile_size: i32, tileset_width: i32) -> Rectangle {
            let cols = tileset_width / tile_size;
            let x = (tile_id % cols) * tile_size;
            let y = (tile_id / cols) * tile_size;

            Rectangle {
                x: x as f32,
                y: y as f32,
                width: tile_size as f32,
                height: tile_size as f32,
            }
    }
}

impl Scene for MazeScene {
    fn on_enter(&mut self, rl: &mut RaylibHandle, data: &mut GameData) {
        self.map = load_map(&self.map_path);
        self.tile_size = self.map.tile_size_px;

        // Load texture using the thread from GameData
        if let Some(ref thread) = data.thread {
            self.tileset = Some(
                rl.load_texture(thread, "assets/tileset0.png")
                    .expect("Failed to load tileset")
            );
        }

        for e in &self.map.entities {
            if e.kind == "player" {
                self.player_x = e.x;
                self.player_y = e.y;
            }
        }
    }



    fn handle_input(&mut self, rl: &mut RaylibHandle, _data: &mut GameData) -> SceneSwitch {
        // Handle player movement with arrow keys or WASD
        let mut new_x = self.player_x;
        let mut new_y = self.player_y;
        
        if rl.is_key_pressed(KeyboardKey::KEY_RIGHT) || rl.is_key_pressed(KeyboardKey::KEY_D) {
            new_x += 1;
        }
        if rl.is_key_pressed(KeyboardKey::KEY_LEFT) || rl.is_key_pressed(KeyboardKey::KEY_A) {
            if new_x > 0 {
                new_x -= 1;
            }
        }
        if rl.is_key_pressed(KeyboardKey::KEY_DOWN) || rl.is_key_pressed(KeyboardKey::KEY_S) {
            new_y += 1;
        }
        if rl.is_key_pressed(KeyboardKey::KEY_UP) || rl.is_key_pressed(KeyboardKey::KEY_W) {
            if new_y > 0 {
                new_y -= 1;
            }
        }
        
        // Check if the move is valid and update position
        if self.is_valid_move(new_x, new_y) {
            self.player_x = new_x;
            self.player_y = new_y;
        }
        
        SceneSwitch::None
    }

    fn update(&mut self, _dt: f32, data: &mut GameData) -> SceneSwitch {
        // Check if player has reached the goal
        for e in &self.map.entities {
            if e.kind == "goal" && e.x == self.player_x && e.y == self.player_y {
                // Add points for completing the maze
                data.score();
                return SceneSwitch::Push(Box::new(WinScene));
            }
        }
        
        SceneSwitch::None
    }

    

    fn draw(&self, d: &mut RaylibDrawHandle, data: &mut GameData) {
        d.clear_background(Color::WHITE);
        // Background 
        for y in 0..self.map.grid_h {
            for x in 0..self.map.grid_w {
                let tid = self.map.tiles[y][x];
                if tid >= 0 && is_floor_tile(tid) {
                    self.draw_tile(d, tid, x, y);
                }
            }
        }

        // Structures
        for y in 0..self.map.grid_h {
            for x in 0..self.map.grid_w {
                let tid = self.map.tiles[y][x];
                if tid >= 0 && is_wall_tile(tid) {
                    self.draw_tile(d, tid, x, y);
                }
            }
        }

        // Entities
        for e in &self.map.entities {
            let screen_x = (e.x as i32) * self.tile_size;
            let screen_y = (e.y as i32) * self.tile_size;

            match e.kind.as_str() {
                "player" => {
                    d.draw_circle(
                        screen_x + self.tile_size / 2,
                        screen_y + self.tile_size / 2,
                        self.tile_size as f32 * 0.4,
                        Color::BLUE,
                    );
                }
                "goal" => {
                    d.draw_rectangle(
                        screen_x,
                        screen_y,
                        self.tile_size,
                        self.tile_size,
                        Color::GOLD,
                    );
                }
                _ => {}
            }
        }


       
        d.draw_text(
            &format!("Score: {}", data.points),
            10,
            data.screen_height - 24,
            20,
            Color::WHITE,
        );
        
        // // Draw score
        // let message = format!("Score: {}", data.points);
        // d.draw_text(message.as_str(), 10, data.screen_height - 25, 20, Color::BLACK);
    }

    fn on_exit(&mut self, _rl: &mut RaylibHandle, _data: &mut GameData) {}
}
