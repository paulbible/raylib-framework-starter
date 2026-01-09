//! The data for each game session. 
//! 
//! You could also store data associated with each human player here.
//! We could also store the player's gamepad_id here.

pub struct GameData {
    pub points: u32,
    pub screen_width: i32,
    pub screen_height: i32,
}

impl GameData {
    pub fn new(width: i32, heigth: i32) -> Self {
        Self {
            points: 0,
            screen_width: width,
            screen_height: heigth
        }
    }

    /// add one to the player's total points.
    pub fn score(&mut self) {
        self.points += 1;
    }
}