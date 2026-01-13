//! A scene to show a menu
//! 
//! 
use raylib::prelude::*;
// use rand::{self, Rng};

use crate::game_data::GameData;
use crate::game_scene::GameScene;
use crate::scenes::{Scene,SceneSwitch}; 
use crate::utils::*;

/// A start screen or menu screen scene
pub struct MenuScene;

impl Scene for MenuScene {
    fn on_enter(&mut self, _rl: &mut RaylibHandle, _data: &mut GameData) {}

    fn handle_input(&mut self, _rl: &mut RaylibHandle, data: &mut GameData) -> SceneSwitch {

        if _rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) {
            let click = _rl.get_mouse_position();
            let rectangle = Rectangle::new(200.0, 200.0, 300.0, 150.0);
            if  check_collision_point_rect(&click, &rectangle) {
                println!("click");
                return SceneSwitch::Push(Box::new(GameScene::new(5, data.screen_width, data.screen_height)));
            }
        }
        
        SceneSwitch::None
    }

    fn update(&mut self, _dt: f32, _data: &mut GameData) -> SceneSwitch {
        SceneSwitch::None

    }

    fn draw(&self, d: &mut RaylibDrawHandle, _data: &mut GameData) {
        d.clear_background(Color::WHITE);
        
        d.draw_rectangle(200, 200, 300, 150, Color::RED);
        d.draw_text("Click here", 210, 205, 20, Color::BLACK);
    }

    fn on_exit(&mut self, _rl: &mut RaylibHandle, _data: &mut GameData) {}
}


/// A win screen scene
pub struct WinScene;

impl Scene for WinScene {
    fn on_enter(&mut self, _rl: &mut RaylibHandle, _data: &mut GameData) {}

    fn handle_input(&mut self, _rl: &mut RaylibHandle, _data: &mut GameData) -> SceneSwitch {

        if _rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) {
            let click = _rl.get_mouse_position();
            let rectangle = Rectangle::new(200.0, 200.0, 300.0, 150.0);
            if  check_collision_point_rect(&click, &rectangle) {
                println!("click");
                // close the program
                // return SceneSwitch::Quit;
                return SceneSwitch::Pop;
            }
        }
        
        SceneSwitch::None
    }

    fn update(&mut self, _dt: f32, _data: &mut GameData) -> SceneSwitch {
        SceneSwitch::None

    }

    fn draw(&self, d: &mut RaylibDrawHandle, _data: &mut GameData) {
        d.clear_background(Color::WHITE);
        
        d.draw_rectangle(200, 200, 300, 150, Color::BLUE);
        d.draw_text("You Win!", 210, 205, 20, Color::BLACK);
        let message = format!("Final score: {}", _data.points);
        d.draw_text(message.as_str(), 210, 225, 20, Color::BLACK);
        d.draw_text("Click here to exit.", 210, 250, 20, Color::BLACK);
    }

    fn on_exit(&mut self, _rl: &mut RaylibHandle, _data: &mut GameData) {}
}


pub struct PauseScene;

impl Scene for PauseScene {
    fn on_enter(&mut self, _rl: &mut RaylibHandle, _data: &mut GameData) {}

    fn handle_input(&mut self, _rl: &mut RaylibHandle, _data: &mut GameData) -> SceneSwitch {

        if _rl.is_key_pressed(KeyboardKey::KEY_P) {
            return SceneSwitch::Pop;
        }

        // if _rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) {
        //     let click = _rl.get_mouse_position();
        //     let rectangle = Rectangle::new(200.0, 200.0, 300.0, 150.0);
        //     if  check_collision_point_rect(&click, &rectangle) {
        //         println!("click");
                
        //     }
        // }
        
        SceneSwitch::None
    }

    fn update(&mut self, _dt: f32, _data: &mut GameData) -> SceneSwitch {
        SceneSwitch::None

    }

    fn draw(&self, d: &mut RaylibDrawHandle, _data: &mut GameData) {
        d.clear_background(Color::WHITE);
        
        d.draw_rectangle(200, 200, 300, 150, Color::GRAY);
        // d.draw_text("You Win!", 210, 205, 20, Color::BLACK);
        let message = format!("Current score: {}", _data.points);
        d.draw_text(message.as_str(), 210, 225, 20, Color::BLACK);
        d.draw_text("Press P to resume.", 210, 250, 20, Color::BLACK);
    }

    fn on_exit(&mut self, _rl: &mut RaylibHandle, _data: &mut GameData) {}
}