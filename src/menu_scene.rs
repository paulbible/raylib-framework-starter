//! A scene to show a menu
//! 
//! 
use raylib::prelude::*;
// use rand::{self, Rng};

use crate::game_data::GameData;
use crate::maze_scene::MazeScene;
use crate::scenes::{Scene,SceneSwitch}; 
use crate::utils::*;

/// A start screen or menu screen scene
/// A start screen or menu screen scene
pub struct TitleScene;

impl Scene for TitleScene {
    fn on_enter(&mut self, _rl: &mut RaylibHandle, _data: &mut GameData) {}

    fn handle_input(&mut self, _rl: &mut RaylibHandle, _data: &mut GameData) -> SceneSwitch {
        if _rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) {
            let click = _rl.get_mouse_position();
            // Button rectangle: centered in bottom half (480-960)
            let button_rect = Rectangle::new(490.0, 645.0, 300.0, 150.0);
            if check_collision_point_rect(&click, &button_rect) {
                return SceneSwitch::Push(Box::new(MenuScene));
            }
        }
        
        SceneSwitch::None
    }

    fn update(&mut self, _dt: f32, _data: &mut GameData) -> SceneSwitch {
        SceneSwitch::None
    }

    fn draw(&self, d: &mut RaylibDrawHandle, data: &mut GameData) {
        d.clear_background(Color::WHITE);
        
        // Draw title: centered in top half (0-480)
        d.draw_text("Dungeon Diver", 385, 215, 70, Color::BLACK);
        
        // Draw "Start" button: centered in bottom half (480-960)
        d.draw_rectangle(490, 645, 300, 150, Color::GREEN);
        d.draw_text("Start", 600, 700, 30, Color::WHITE);  // Centered inside button
    }

    fn on_exit(&mut self, _rl: &mut RaylibHandle, _data: &mut GameData) {}
}

pub struct MenuScene;

impl Scene for MenuScene {
    fn on_enter(&mut self, _rl: &mut RaylibHandle, _data: &mut GameData) {}

    fn handle_input(&mut self, _rl: &mut RaylibHandle, data: &mut GameData) -> SceneSwitch {

        if _rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) {
            let click = _rl.get_mouse_position();
            let rectangle = Rectangle::new(200.0, 200.0, 150.0, 50.0);
            if  check_collision_point_rect(&click, &rectangle) {
                println!("clicked on stage");
                return SceneSwitch::Push(Box::new(MazeScene::from_map("assets/maps/mapTest.json".to_string())))


            }
        }
        
        SceneSwitch::None
    }

    fn update(&mut self, _dt: f32, _data: &mut GameData) -> SceneSwitch {
        SceneSwitch::None

    }

    fn draw(&self, d: &mut RaylibDrawHandle, _data: &mut GameData) {
        d.clear_background(Color::WHITE);
        d.draw_text("Dungeon Stages", 450, 95, 50, Color::BLACK);
        d.draw_rectangle(200, 200, 150, 50, Color::GREEN);
        d.draw_text("Stage I", 235, 215, 20, Color::WHEAT);
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
                return SceneSwitch::Pop;
                //return SceneSwitch::Quit;
            }
        }
        
        SceneSwitch::None
    }

    fn update(&mut self, _dt: f32, _data: &mut GameData) -> SceneSwitch {
        SceneSwitch::None

    }

    fn draw(&self, d: &mut RaylibDrawHandle, _data: &mut GameData) {
        d.clear_background(Color::WHITE);
        
        d.draw_rectangle(200, 200, 300, 150, Color::GREEN);
        d.draw_text("Win", 210, 205, 20, Color::BLACK);
        let message = format!("Final score: {}", _data.points);
        d.draw_text(message.as_str(), 210, 225, 20, Color::BLACK);
        d.draw_text("Click to quit.", 210, 250, 20, Color::BEIGE);
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
        //         // close the program
        //         return SceneSwitch::Quit;
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
        d.draw_text("Paused", 210, 205, 20, Color::WHITE);
        let message = format!("Current score: {}", _data.points);
        d.draw_text(message.as_str(), 210, 225, 20, Color::WHEAT);
        d.draw_text("Press P to resume.", 210, 250, 20, Color::WHITE);
    }

    fn on_exit(&mut self, _rl: &mut RaylibHandle, _data: &mut GameData) {}
}      