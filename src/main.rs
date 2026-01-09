use raylib::prelude::*;

use raylib_framework_testing::game_data::GameData;
use raylib_framework_testing::menu_scene::MenuScene;
use raylib_framework_testing::scenes::{Scene, SceneSwitch};

use std::time::Instant;

fn main() {
    let width: i32 = 640;
    let height: i32 = 480;
    let (mut rl, thread) = raylib::init()
        .size(width, height)
        .title("Raylib Framework Demo")
        .build();


    // create the game data. This prepresents data associated with the human player.
    let mut game_data = GameData::new(width, height);

    // store scenes in a Vec. Box<dyn Scene> means a pointer to any type that implements the Scene trait.
    let mut scenes: Vec<Box<dyn Scene>> = vec![Box::new(MenuScene)];

    // A variable for the time to calculate update steps in the game. Use for physics and animation.
    let mut last_time = Instant::now();
    let mut keep_playing = true;
     
    // the main game / draw loop 
    while !rl.window_should_close() && keep_playing {
        // update timing.
        let temp = Instant::now();
        let delta = (temp - last_time).as_secs_f32();
        last_time = temp;

        // Below is a general framework for working with a game. Look at the Scene trait for some more information
        //
        // 1) preprocessing / setup. This was named on_enter. Called when the start is first started.
        // --- Main loop ---
        // 2) handle user input. Get the player's intent.
        // 3) update the world / do simulation processing.
        // 4) draw the game elements
        // --- End of main loop ---
        // 5) postprocessing / clean up. This was named on_exit. Do any necessary clean up

        let mut the_scene = scenes.last_mut().unwrap();

        // handle user input. This 
        //let result = scenes.last_mut().unwrap().handle_input(&mut rl, &mut game_data);
        let result = the_scene.handle_input(&mut rl, &mut game_data);
        match result {
            SceneSwitch::Push(new_scene) => {
                println!("got scene");
                scenes.push(new_scene);
            },
            SceneSwitch::Quit => keep_playing = false,
            _ => ()
        }


        let mut the_scene = scenes.last_mut().unwrap();
        //let result = scenes.last_mut().unwrap().update(delta, &mut game_data);
        let result = the_scene.update(delta, &mut game_data);
        match result {
            SceneSwitch::Push(new_scene) => {
                println!("got scene");
                scenes.push(new_scene);
            },
            SceneSwitch::Quit => keep_playing = false,
            _ => ()
        }


        // Draw
        let mut the_scene = scenes.last().unwrap();
        let mut d = rl.begin_drawing(&thread); 
        the_scene.draw(&mut d, &mut  game_data);

    }
}