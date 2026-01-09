//! Traits for scenes and the scene switch signals.
//! 
use raylib::prelude::*;

use crate::game_data::GameData;
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

