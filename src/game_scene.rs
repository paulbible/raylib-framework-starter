//! The core game play scene
//! 
//! This represents the chase game. Here we store information about the game world and the player's "character".

use raylib::prelude::*;

use crate::menu_scene::{WinScene, PauseScene};
use crate::scenes::{Scene, SceneSwitch};
use crate::game_data::GameData;
use crate::utils::*;

pub struct GameScene {
    points: Vec<Vector2>,
    player_position: Vector2,
    player_direction: Vector2,
    player_speed: f32
}

impl GameScene {
    pub fn new(n: usize, width: i32, height: i32) -> Self {
        let mut points = Vec::new();
        for _ in 0..n {
            points.push(random_point(width, height));
        }
        Self { 
            points: points,
            player_position: Vector2::new((width/2) as f32, (height/2) as f32),
            player_direction: Vector2::zero(),
            player_speed: 300.0
        }
    }
}

impl Scene for GameScene {
    fn on_enter(&mut self, _rl: &mut RaylibHandle, _data: &mut GameData) {
        
    }

    fn handle_input(&mut self, _rl: &mut RaylibHandle, _data: &mut GameData) -> SceneSwitch {
        
        // set the intention to move in the given direction.
        let mut direction = Vector2::zero();
        if _rl.is_key_down(KeyboardKey::KEY_A) || 
            _rl.is_key_down(KeyboardKey::KEY_LEFT) 
        {
            direction += Vector2::new(-1.0, 0.0);
        }
        
        if _rl.is_key_down(KeyboardKey::KEY_D) || 
            _rl.is_key_down(KeyboardKey::KEY_RIGHT) 
        {
            direction += Vector2::new(1.0, 0.0);
        }

        if _rl.is_key_down(KeyboardKey::KEY_W) || 
            _rl.is_key_down(KeyboardKey::KEY_UP) 
        {
            direction += Vector2::new(0.0, -1.0);
        }

        if _rl.is_key_down(KeyboardKey::KEY_S) || 
            _rl.is_key_down(KeyboardKey::KEY_DOWN) 
        {
            direction += Vector2::new(0.0, 1.0);
        }
        if _rl.is_key_pressed(KeyboardKey::KEY_P) {
            return SceneSwitch::Push(Box::new(PauseScene));
        }

        direction.normalize();

        self.player_direction = direction;

        SceneSwitch::None
    }

    fn update(&mut self, _dt: f32, data: &mut GameData) -> SceneSwitch {

        // update position of player, deal with collisions (later ...)
        let speed_delta = self.player_speed * _dt;
        self.player_position = self.player_position + self.player_direction * speed_delta;


        if let Some(last) = self.points.last() {
            // remove the last point.
            if last.distance_to(self.player_position) < 25.0 {
                self.points.pop();
                data.score();
            } 
        } else {
            println!("Deal with win condition, send new scene");
            return SceneSwitch::Replace(Box::new(WinScene));
        }


        SceneSwitch::None
    }

    fn draw(&self, d: &mut RaylibDrawHandle, data: &mut GameData){
        d.clear_background(Color::WHITE);

        // Draw player
        d.draw_circle(self.player_position.x as i32,
             self.player_position.y as i32, 
             15.0, 
             Color::BLACK);
        
        // Draw last point in the vector
        if let Some(last) = self.points.last() {
            d.draw_circle(last.x as i32,
             last.y as i32, 
            20.0, 
             Color::BLUE);
        }

        // Draw score based on game data
        let message = format!("Score: {}", data.points);
        d.draw_text(message.as_str(), 10, data.screen_height - 25, 20, Color::BLACK);
    }

    fn on_exit(&mut self, _rl: &mut RaylibHandle, _data: &mut GameData) {}
}