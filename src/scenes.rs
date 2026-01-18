//! Traits for scenes and the scene switch signals.
//! 
use raylib::prelude::*;

use crate::{game_data::GameData, scenes};
///
/// The SceneSwitch enum was conceived with the help of ChatGPT 5.2
/// 
/// These values will signal to the manage that we need to change / update the scene
pub enum SceneSwitch {
    None,
    Push(Box<dyn Scene>),
    Replace(Box<dyn Scene>),
    Pop,
    Quit,
}

///
/// The Scene trait was conceived with the help of ChatGPT 5.2
/// 
/// A manager will call these methods to implement a typical videogame / interactive program loop.
/// 
/// The leading underscore tells the compiler not to complain (warn) if that variable is not read. 
pub trait Scene {
    
    /// called when the scene is first started.
    fn on_enter(&mut self, _rl: &mut RaylibHandle, _data: &mut GameData) {}

    /// collects the player's intent from the controller / keyboard / input hardware.
    fn handle_input(&mut self, _rl: &mut RaylibHandle, _data: &mut GameData) -> SceneSwitch {
        SceneSwitch::None
    }

    /// update / evolve the scene according to a time step dt.
    fn update(&mut self, _dt: f32, _data: &mut GameData) -> SceneSwitch {
        SceneSwitch::None
    }

    /// draw the scene elements. This should be very simple code that only draws using the RaylibDrawHandle
    fn draw(&self, d: &mut RaylibDrawHandle, data: &mut GameData);

    /// called when the scene is finished. Do any clean up that is needed when the game ends (free textures or other data).
    /// Rust may take care of most of the memory clean up, but releasing GPU memory might go here.
    fn on_exit(&mut self, _rl: &mut RaylibHandle, _data: &mut GameData) {}
}


/// SceneManager
/// 
/// This struct controls switching be between different scenes.
pub struct SceneManager {
    scenes: Vec<Box<dyn Scene>>,
    quit: bool,

}

impl SceneManager {
    pub fn new(rl: &mut RaylibHandle, initial: Box<dyn Scene>, data: &mut GameData) -> Self {
        let mut mgr = Self {
            scenes: vec![initial],
            quit: false,
        };
        mgr.scenes.last_mut().unwrap().on_enter(rl, data);
        mgr
    }

    /// handles collecting user input by calling the scene's [`Scene::handle_input`] and does time step updating with [update]
    pub fn update(&mut self, rl: &mut RaylibHandle, dt: f32, data: &mut GameData) {
        if let Some(scene) = self.scenes.last_mut() {
            let switch = scene.handle_input(rl, data);
            self.apply_switch(switch, rl, data);
        }

        if let Some(scene) = self.scenes.last_mut() {
            let switch = scene.update(dt, data);
            self.apply_switch(switch, rl, data);
        }
    }

    // calls the current scene's [draw] method
    pub fn draw(&self, d: &mut RaylibDrawHandle, data: &mut GameData) {
        if let Some(scene) = self.scenes.last() {
            scene.draw(d, data);
        }
    }

    // applies a switch returned by either the [handle_input] method or the [update] method.
    pub fn apply_switch(&mut self, switch: SceneSwitch, rl: &mut RaylibHandle, data: &mut GameData) {
        match switch {
            SceneSwitch::None => {},
            SceneSwitch::Push(mut scene) => {
                scene.on_enter(rl, data);
                self.scenes.push(scene);
            },
            SceneSwitch::Replace(mut scene) => {
                if let Some(mut old_scene) = self.scenes.pop() {
                    old_scene.on_exit(rl, data);
                }
                scene.on_enter(rl, data);
                self.scenes.push(scene);
            }
            SceneSwitch::Pop => {
                if let Some(mut old_scene) = self.scenes.pop() {
                    old_scene.on_exit(rl, data);
                }
            },
            SceneSwitch::Quit => {
                self.quit = true;
            }
        }
    }

    pub fn should_quit(&self) -> bool {
        self.quit || self.scenes.is_empty() 
    }
}