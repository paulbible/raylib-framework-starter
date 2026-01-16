//! The data for each game session. 
//! 
//! You could also store data associated with each human player here.
//! We could also store the player's gamepad_id here.

use raylib::prelude::*;

pub struct GameData {
    pub points: u32,
    pub screen_width: i32,
    pub screen_height: i32,
    pub thread: Option<RaylibThread>, 
}

impl GameData {
    pub fn new(width: i32, heigth: i32) -> Self {
        Self {
            points: 0,
            screen_width: width,
            screen_height: heigth,
            thread: None,
        }
    }
    
    pub fn set_thread(&mut self, thread: RaylibThread) {
        self.thread = Some(thread);
    }

    /// add one to the player's total points.
    pub fn score(&mut self) {
        self.points += 1;
    }
}