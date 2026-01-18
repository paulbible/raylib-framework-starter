//! The data for each game session. 
//! 
//! You could also store data associated with each human player here.
//! We could also store the player's gamepad_id here.

use raylib::prelude::*;
use std::time::Instant;

pub struct GameData {
    pub points: u32,
    pub screen_width: i32,
    pub screen_height: i32,
    pub thread: Option<RaylibThread>,
    // Timing for level completion
    pub level_start_time: Option<Instant>,
    pub level_completion_time: Option<Instant>,
}

impl GameData {
    pub fn new(width: i32, heigth: i32) -> Self {
        Self {
            points: 0,
            screen_width: width,
            screen_height: heigth,
            thread: None,
            level_start_time: None,
            level_completion_time: None,
        }
    }
    
    pub fn set_thread(&mut self, thread: RaylibThread) {
        self.thread = Some(thread);
    }

    /// add one to the player's total points.
    pub fn score(&mut self) {
        self.points += 1;
    }
    
    /// Start timing a level
    pub fn start_level_timer(&mut self) {
        self.level_start_time = Some(Instant::now());
        self.level_completion_time = None;
    }
    
    /// Record level completion time
    pub fn complete_level(&mut self) {
        self.level_completion_time = Some(Instant::now());
    }
    
    /// Get elapsed time in seconds (returns None if level hasn't started or completed)
    pub fn get_elapsed_time(&self) -> Option<f32> {
        if let (Some(start), Some(completion)) = (self.level_start_time, self.level_completion_time) {
            Some((completion - start).as_secs_f32())
        } else {
            None
        }
    }
}