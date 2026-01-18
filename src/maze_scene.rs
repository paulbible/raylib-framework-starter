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
    
    // Camera system
    camera: Camera2D,
    fov_radius: i32, // tiles
    
    // Tick-based game logic
    tick_timer: f32,
    tick_rate: f32, // seconds per tick
    
    // Queued movement for tick system
    queued_move: Option<(usize, usize)>,
    
    // Gamepad input tracking
    last_gamepad_direction: Option<(i32, i32)>, // (x_dir, y_dir) - tracks last discrete direction
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
            // Initialize camera centered on origin (will be updated in on_enter)
            camera: Camera2D {
                target: Vector2::zero(),
                offset: Vector2::zero(),
                rotation: 0.0,
                zoom: 1.0,
            },
            fov_radius: 7, 
            tick_timer: 0.0,
            tick_rate: 0.15, // ~6.6 ticks per second (150ms per tick)
            queued_move: None,
            last_gamepad_direction: None,
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
    
    /// Check if a tile is within the player's field of view
    fn in_fov(&self, x: usize, y: usize) -> bool {
        // Bounds check first
        if x >= self.map.grid_w || y >= self.map.grid_h {
            return false;
        }
        
        // Calculate squared distance
        let dx = x as i32 - self.player_x as i32;
        let dy = y as i32 - self.player_y as i32;
        let dist_squared = dx * dx + dy * dy;
        let radius_squared = self.fov_radius * self.fov_radius;
        
        dist_squared <= radius_squared
    }
    
    /// Calculate visible tile bounds for optimized drawing
    /// Returns (min_x, max_x, min_y, max_y) clamped to map bounds
    fn get_visible_bounds(&self) -> (usize, usize, usize, usize) {
        let min_x = self.player_x.saturating_sub(self.fov_radius as usize);
        let max_x = (self.player_x + self.fov_radius as usize + 1).min(self.map.grid_w);
        let min_y = self.player_y.saturating_sub(self.fov_radius as usize);
        let max_y = (self.player_y + self.fov_radius as usize + 1).min(self.map.grid_h);
        
        (min_x, max_x, min_y, max_y)
    }
    
    /// Update camera to follow player (centered on screen)
    fn update_camera(&mut self, data: &GameData) {
        // Convert player tile position to world pixel position (center of tile)
        self.camera.target = Vector2::new(
            (self.player_x as i32 * self.tile_size + self.tile_size / 2) as f32,
            (self.player_y as i32 * self.tile_size + self.tile_size / 2) as f32,
        );
        
        // Offset camera so player appears centered on screen
        self.camera.offset = Vector2::new(
            (data.screen_width / 2) as f32,
            (data.screen_height / 2) as f32,
        );
    }
    
    /// Process player movement on game tick
    fn update_player(&mut self) {
        if let Some((new_x, new_y)) = self.queued_move.take() {
            if self.is_valid_move(new_x, new_y) {
                self.player_x = new_x;
                self.player_y = new_y;
            }
        }
    }
    
    /// Update enemy AI on game tick (placeholder for future implementation)
    fn update_enemies(&mut self) {
        // Enemies update even outside FOV - simulation is separate from rendering
        // This is where tank/shooter AI would go
        // For now, this is a placeholder
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

        // Initialize player position from map entities
        let mut player_initialized = false;
        for e in &self.map.entities {
            if e.kind == "player" {
                self.player_x = e.x;
                self.player_y = e.y;
                player_initialized = true;
                break;
            }
        }
        
        // If no player entity found, try to find first valid floor tile (fool checks)
        if !player_initialized {
            'outer: for y in 0..self.map.grid_h {
                for x in 0..self.map.grid_w {
                    if self.is_valid_move(x, y) {
                        self.player_x = x;
                        self.player_y = y;
                        break 'outer;
                    }
                }
            }
        }
        
        // Filter out player entities from map (player is now separate)
        self.map.entities.retain(|e| e.kind != "player");
        
        // Initialize camera position
        self.update_camera(data);
        
        // Start level timer when entering the maze
        data.start_level_timer();
    }



    fn handle_input(&mut self, rl: &mut RaylibHandle, _data: &mut GameData) -> SceneSwitch {
        // Queue movement for tick-based updates (only queue if no move is already queued)
        if self.queued_move.is_none() {
            let mut new_x = self.player_x;
            let mut new_y = self.player_y;
            let mut movement_queued = false;
            
            // ===== KEYBOARD INPUT =====
            if rl.is_key_down(KeyboardKey::KEY_RIGHT) || rl.is_key_down(KeyboardKey::KEY_D) {
                new_x = new_x.saturating_add(1).min(self.map.grid_w.saturating_sub(1));
                movement_queued = true;
            }
            if rl.is_key_down(KeyboardKey::KEY_LEFT) || rl.is_key_down(KeyboardKey::KEY_A) {
                new_x = new_x.saturating_sub(1);
                movement_queued = true;
            }
            if rl.is_key_down(KeyboardKey::KEY_DOWN) || rl.is_key_down(KeyboardKey::KEY_S) {
                new_y = new_y.saturating_add(1).min(self.map.grid_h.saturating_sub(1));
                movement_queued = true;
            }
            if rl.is_key_down(KeyboardKey::KEY_UP) || rl.is_key_down(KeyboardKey::KEY_W) {
                new_y = new_y.saturating_sub(1);
                movement_queued = true;
            }
            
            // ===== GAMEPAD INPUT =====
            // Check if gamepad is available
            if rl.is_gamepad_available(0) {
                let x_axis = rl.get_gamepad_axis_movement(0, GamepadAxis::GAMEPAD_AXIS_LEFT_X);
                let y_axis = rl.get_gamepad_axis_movement(0, GamepadAxis::GAMEPAD_AXIS_LEFT_Y);
                
                // Convert analog stick input to discrete directions
                // Threshold: stick must be pushed at least 0.5 to register input (deadzone)
                let deadzone = 0.5;
                
                // Determine discrete direction from analog input
                // Prioritize the axis with greater magnitude for diagonal movement
                let abs_x = x_axis.abs();
                let abs_y = y_axis.abs();
                
                if abs_x > deadzone || abs_y > deadzone {
                    // Determine which direction to move (prioritize stronger axis)
                    let mut gamepad_x_dir = 0;
                    let mut gamepad_y_dir = 0;
                    
                    if abs_x > abs_y {
                        // Horizontal movement takes priority
                        gamepad_x_dir = if x_axis > 0.0 { 1 } else { -1 };
                    } else if abs_y > abs_x {
                        // Vertical movement takes priority
                        gamepad_y_dir = if y_axis > 0.0 { 1 } else { -1 };
                    } else {
                        // Equal magnitude - allow diagonal movement
                        if abs_x > deadzone {
                            gamepad_x_dir = if x_axis > 0.0 { 1 } else { -1 };
                        }
                        if abs_y > deadzone {
                            gamepad_y_dir = if y_axis > 0.0 { 1 } else { -1 };
                        }
                    }
                    
                    // Apply gamepad movement (works like keyboard - queues every frame when stick is pushed)
                    if gamepad_x_dir != 0 {
                        new_x = if gamepad_x_dir > 0 {
                            new_x.saturating_add(1).min(self.map.grid_w.saturating_sub(1))
                        } else {
                            new_x.saturating_sub(1)
                        };
                        movement_queued = true;
                    }
                    if gamepad_y_dir != 0 {
                        new_y = if gamepad_y_dir > 0 {
                            new_y.saturating_add(1).min(self.map.grid_h.saturating_sub(1))
                        } else {
                            new_y.saturating_sub(1)
                        };
                        movement_queued = true;
                    }
                } else {
                    // Stick is in deadzone - reset tracking
                    self.last_gamepad_direction = None;
                }
            }
    
            // Only queue if position changed
            if movement_queued && (new_x != self.player_x || new_y != self.player_y) {
                self.queued_move = Some((new_x, new_y));
            }
        }
        
        SceneSwitch::None
    }

    fn update(&mut self, dt: f32, data: &mut GameData) -> SceneSwitch {
        // Update camera every frame
        self.update_camera(data);
        
        // Tick-based game logic
        // Accumulate time until we reach tick_rate, then process one game tick
        self.tick_timer += dt;
        
        // Process game tick when timer exceeds tick_rate
        if self.tick_timer >= self.tick_rate {
            self.tick_timer = 0.0;
            
            // Update player movement (grid-locked, tick-based)
            self.update_player();
            
            // Update enemy AI
            self.update_enemies();
        }
        
        // Check if player has reached the goal 
        for e in &self.map.entities {
            if e.kind == "goal" && e.x == self.player_x && e.y == self.player_y {
                // Add points for completing the maze
                data.score();
                // Record completion time
                data.complete_level();
                return SceneSwitch::Replace(Box::new(WinScene));
            }
        }
        
        SceneSwitch::None
    }

    

    fn draw(&self, d: &mut RaylibDrawHandle, data: &mut GameData) {
        d.clear_background(Color::BLACK);
        
        // Begin 2D camera mode
        let mut d2d = d.begin_mode2D(self.camera);
        
        // only draw tiles in FOV
        let (min_x, max_x, min_y, max_y) = self.get_visible_bounds();
        
        // ===== FLOOR LAYER =====
        // Only iterate over visible tiles
        for y in min_y..max_y {
            for x in min_x..max_x {
                // skip tiles outside circular FOV
                if !self.in_fov(x, y) {
                    continue;
                }
                
                let tid = self.map.tiles[y][x];
                // Draw floor tiles, or any non-wall tile as floor
                if tid >= 0 && (is_floor_tile(tid) || !is_wall_tile(tid)) {
                    self.draw_tile(&mut d2d, tid, x, y);
                }
            }
        }
        
        // ===== WALL LAYER =====
        // Only iterate over visible tiles
        for y in min_y..max_y {
            for x in min_x..max_x {
                // skip tiles outside circular FOV
                if !self.in_fov(x, y) {
                    continue;
                }
                
                let tid = self.map.tiles[y][x];
                if tid >= 0 && is_wall_tile(tid) {
                    self.draw_tile(&mut d2d, tid, x, y);
                }
            }
        }
        
        // ===== ENTITIES LAYER =====
        // Draw entities only if they're in FOV
        for e in &self.map.entities {
            // FOV culling: skip entities outside circular FOV
            if !self.in_fov(e.x, e.y) {
                continue;
            }
            
            // Convert tile coordinates to world pixel coordinates
            let px = (e.x as i32 * self.tile_size) as f32;
            let py = (e.y as i32 * self.tile_size) as f32;
            
            match e.kind.as_str() {
                "goal" => {
                    d2d.draw_rectangle(
                        px as i32,
                        py as i32,
                        self.tile_size,
                        self.tile_size,
                        Color::GOLD,
                    );
                }
                "tank" => {
                    // Placeholder for tank rendering
                    d2d.draw_circle(
                        (px + self.tile_size as f32 / 2.0) as i32,
                        (py + self.tile_size as f32 / 2.0) as i32,
                        self.tile_size as f32 * 0.3,
                        Color::RED,
                    );
                }
                "shooter" => {
                    // Placeholder for shooter rendering
                    d2d.draw_circle(
                        (px + self.tile_size as f32 / 2.0) as i32,
                        (py + self.tile_size as f32 / 2.0) as i32,
                        self.tile_size as f32 * 0.25,
                        Color::ORANGE,
                    );
                }
                _ => {}
            }
        }
        
        // ===== PLAYER (always visible, drawn on top) =====
        // Player is always drawn, even if outside FOV (shouldn't happen, but safe)
        let player_px = self.player_x as i32 * self.tile_size + self.tile_size / 2;
        let player_py = self.player_y as i32 * self.tile_size + self.tile_size / 2;
        d2d.draw_circle(
            player_px,
            player_py,
            self.tile_size as f32 * 0.4,
            Color::BLUE,
        );
        
        // End 2D camera mode
        drop(d2d);
        
        // ===== UI LAYER (screen space, not affected by camera) =====
        d.draw_text(
            &format!("Score: {}", data.points),
            10,
            data.screen_height - 24,
            20,
            Color::WHITE,
        );
    }

    fn on_exit(&mut self, _rl: &mut RaylibHandle, _data: &mut GameData) {}
}

